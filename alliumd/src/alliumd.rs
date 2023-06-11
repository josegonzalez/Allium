use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;
use common::constants::{ALLIUMD_STATE, ALLIUM_GAME_INFO, ALLIUM_LAUNCHER, ALLIUM_MENU};
use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use tracing::{debug, info, trace};

use common::platform::{DefaultPlatform, Key, KeyEvent, Platform};

#[cfg(unix)]
use {
    futures::future::{Fuse, FutureExt},
    nix::sys::signal::kill,
    nix::sys::signal::Signal,
    nix::unistd::Pid,
    std::os::unix::process::CommandExt,
    tokio::signal::unix::SignalKind,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AlliumD<P: Platform> {
    #[serde(skip)]
    platform: P,
    #[serde(skip, default = "spawn_main")]
    main: Child,
    #[serde(skip)]
    menu: Option<Child>,
    #[serde(skip)]
    is_menu_pressed: bool,
    #[serde(skip)]
    is_menu_pressed_alone: bool,
    volume: i32,
    brightness: u8,
}

fn spawn_main() -> Child {
    try_load_game().unwrap_or_else(|| {
        let path = Path::new(&*ALLIUM_GAME_INFO);
        if path.exists() {
            info!("no game info found, launching menu");
            fs::remove_file(path).unwrap();
        } else {
            info!("failed to load game, launching menu");
        }
        Command::new(ALLIUM_LAUNCHER.as_path()).spawn().unwrap()
    })
}

fn try_load_game() -> Option<Child> {
    if ALLIUM_GAME_INFO.exists() {
        let game_info = fs::read_to_string(ALLIUM_GAME_INFO.as_path()).ok()?;
        let mut split = game_info.split('\n');
        let _name = split.next();
        let core = split.next().and_then(|path| PathBuf::from_str(path).ok())?;
        Command::new(core).args(split).spawn().ok()
    } else {
        None
    }
}

impl AlliumD<DefaultPlatform> {
    pub fn new() -> Result<AlliumD<DefaultPlatform>> {
        let platform = DefaultPlatform::new()?;
        let brightness = platform.get_brightness()?;

        Ok(AlliumD {
            platform,
            main: spawn_main(),
            menu: None,
            is_menu_pressed: false,
            is_menu_pressed_alone: false,
            volume: 0,
            brightness,
        })
    }

    pub async fn run_event_loop(&mut self) -> Result<()> {
        info!("running Alliumd");

        self.platform.set_volume(self.volume)?;

        #[cfg(unix)]
        {
            let mut sighup = tokio::signal::unix::signal(SignalKind::hangup())?;
            let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())?;
            let mut sigquit = tokio::signal::unix::signal(SignalKind::quit())?;
            let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())?;

            loop {
                let menu_terminated = match self.menu.as_mut() {
                    Some(menu) => menu.wait().fuse(),
                    None => Fuse::terminated(),
                };

                tokio::select! {
                    key_event = self.platform.poll() => {
                        if let Some(key_event) = key_event? {
                            self.handle_key_event(key_event).await?;
                        }
                    }
                    _ = self.main.wait() => {
                        info!("main process terminated, restarting");
                        self.main = spawn_main();
                    }
                    _ = menu_terminated => {
                        info!("menu process terminated, resuming game");
                        self.menu = None;
                        #[cfg(unix)]
                        signal(&self.main, Signal::SIGCONT)?;
                    }
                    _ = sighup.recv() => self.handle_quit()?,
                    _ = sigint.recv() => self.handle_quit()?,
                    _ = sigquit.recv() => self.handle_quit()?,
                    _ = sigterm.recv() => self.handle_quit()?,
                }
            }
        }

        #[cfg(not(unix))]
        loop {
            tokio::select! {
                key_event = self.platform.poll() => {
                    if let Some(key_event) = key_event? {
                        self.handle_key_event(key_event).await?;
                    }
                }
            }
        }
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        trace!(
            "menu: {:?}, main: {:?}, ingame: {}",
            self.menu.as_ref().map(|c| c.id()),
            self.main.id(),
            self.is_ingame()
        );
        if matches!(key_event, KeyEvent::Pressed(Key::Menu)) {
            self.is_menu_pressed = true;
            self.is_menu_pressed_alone = true;
        } else if !matches!(key_event, KeyEvent::Released(Key::Menu)) {
            self.is_menu_pressed_alone = false;
        }
        match key_event {
            KeyEvent::Pressed(Key::VolDown) | KeyEvent::Autorepeat(Key::VolDown) => {
                if self.is_menu_pressed {
                    self.add_brightness(-5)?;
                } else {
                    self.add_volume(-1)?
                }
            }
            KeyEvent::Pressed(Key::VolUp) | KeyEvent::Autorepeat(Key::VolUp) => {
                if self.is_menu_pressed {
                    self.add_brightness(5)?;
                } else {
                    self.add_volume(1)?
                }
            }
            KeyEvent::Autorepeat(Key::Power) => {
                self.save()?;
                if self.is_ingame() {
                    if self.menu.is_some() {
                        #[cfg(unix)]
                        signal(&self.main, Signal::SIGCONT)?;
                    }
                    #[cfg(unix)]
                    signal(&self.main, Signal::SIGTERM)?;
                    self.main.wait().await?;
                }
                #[cfg(unix)]
                {
                    std::process::Command::new("sync").spawn()?;
                    std::process::Command::new("poweroff").exec();
                }
            }
            KeyEvent::Released(Key::Menu) => {
                self.is_menu_pressed = false;
                if self.is_ingame() && self.is_menu_pressed_alone {
                    self.is_menu_pressed_alone = false;
                    if let Some(menu) = &mut self.menu {
                        terminate(menu).await?;
                    } else {
                        #[cfg(unix)]
                        signal(&self.main, Signal::SIGSTOP)?;
                        self.menu = Some(Command::new(ALLIUM_MENU.as_path()).spawn()?);
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    #[cfg(unix)]
    fn handle_quit(&mut self) -> Result<()> {
        debug!("terminating, saving state");
        self.save()?;
        Ok(())
    }

    pub fn load() -> Result<AlliumD<DefaultPlatform>> {
        if !ALLIUMD_STATE.exists() {
            debug!("can't find state, creating new");
            Self::new()
        } else {
            debug!("found state, loading from file");
            let json = fs::read_to_string(ALLIUMD_STATE.as_path())?;
            let alliumd: AlliumD<DefaultPlatform> = serde_json::from_str(&json)?;
            Ok(alliumd)
        }
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string(self).unwrap();
        File::create(ALLIUMD_STATE.as_path())?.write_all(json.as_bytes())?;
        Ok(())
    }

    fn is_ingame(&self) -> bool {
        Path::new(&*ALLIUM_GAME_INFO).exists()
    }

    fn add_volume(&mut self, add: i32) -> Result<()> {
        self.volume = (self.volume + add).clamp(0, 20);
        self.platform.set_volume(self.volume)?;
        Ok(())
    }

    fn add_brightness(&mut self, add: i8) -> Result<()> {
        self.brightness = (self.brightness as i8 + add).clamp(0, 100) as u8;
        self.platform.set_brightness(self.brightness)?;
        Ok(())
    }
}

async fn terminate(child: &mut Child) -> Result<()> {
    #[cfg(unix)]
    signal(child, Signal::SIGTERM)?;
    #[cfg(not(unix))]
    child.kill().await?;
    child.wait().await?;
    Ok(())
}

#[cfg(unix)]
fn signal(child: &Child, signal: Signal) -> Result<()> {
    if let Some(pid) = child.id() {
        let pid = Pid::from_raw(pid as i32);
        kill(pid, signal)?;
    }
    Ok(())
}

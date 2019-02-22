extern crate libc;
extern crate chrono;

use timer::{Timer, Guard};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex, Condvar};

mod vjoy;

pub type Button = u8;

pub struct VirtualJoystickConfig {
    button_press_duration_ms: i64
}

impl Default for VirtualJoystickConfig {
    fn default() -> Self {
        VirtualJoystickConfig {
            button_press_duration_ms: 250
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Disabled,
    DriverDoesNotMatch,
    CouldNotAcquire(vjoy::Stat),
    ButtonPressError
}

enum JoystickThreadCommand {
    PressButton(Button),
    ReleaseButton(Button),
    Terminate
}

pub struct VirtualJoystick {
    rID: u32,
    timer: Timer,
    button_release_guards: HashMap<Button, Guard>,
    config: VirtualJoystickConfig,
    tx_command: Sender<JoystickThreadCommand>,
    handle: Option<JoinHandle<()>>
}

impl VirtualJoystick {
    pub fn new(rID: u32, config: Option<VirtualJoystickConfig>) -> Result<Self, Error> {
        use vjoy::Stat;

        if !vjoy::enabled() {
            return Err(Error::Disabled);
        }

        if !vjoy::driver_match() {
            return Err(Error::DriverDoesNotMatch);
        }

        let tuple = Arc::new((Mutex::new(false), Mutex::new(Ok(())), Condvar::new()));
        let tuple2 = tuple.clone();
        let (tx_command, rx_command) = channel();

        let handle = Some(thread::spawn(move || {
            use JoystickThreadCommand::*;

            let &(ref lock_started, ref lock_ok, ref cvar) = &*tuple2;

            let init_ok = match vjoy::get_vjd_status(rID) {
                Stat::Own => {
                    vjoy::reset_vjd(rID);
                    Ok(())
                },
                Stat::Free => {
                    if vjoy::acquire_vjd(rID) {
                        vjoy::reset_vjd(rID);
                        Ok(())
                    } else {
                        Err(Error::CouldNotAcquire(Stat::Free))
                    }
                },
                stat => Err(Error::CouldNotAcquire(stat))
            };

            *lock_started.lock().unwrap() = true;
            if let Err(e) = init_ok {
                *lock_ok.lock().unwrap() = Err(e);
                cvar.notify_one();
                return;
            } else {
                *lock_ok.lock().unwrap() = Ok(());
                cvar.notify_one();
            }

            loop {
                match rx_command.recv().unwrap() {
                    PressButton(btn) => vjoy::set_btn(true, rID, btn),
                    ReleaseButton(btn) => vjoy::set_btn(false, rID, btn),
                    Terminate => {
                        vjoy::relinquish_vjd(rID);
                        break
                    }
                };
            }
        }));

        let &(ref lock_started, ref lock_ok, ref cvar) = &*tuple;
        let mut started = lock_started.lock().unwrap();
        while *started {
            started = cvar.wait(started).unwrap();
        }

        let ok = match *lock_ok.lock().unwrap() {
            Ok(()) => Ok(Self {
                rID: rID,
                timer: Timer::new(),
                button_release_guards: HashMap::new(),
                config: config.unwrap_or(VirtualJoystickConfig::default()),
                tx_command: tx_command,
                handle: handle
            }),
            Err(e) => Err(e)
        };

        ok
    }

    pub fn press_button(&mut self, btn: Button) {
        self.tx_command.send(JoystickThreadCommand::PressButton(btn)).unwrap();

        let rID = self.rID;
        let tx_command = self.tx_command.clone();
        let guard = self.timer.schedule_with_delay(
            chrono::Duration::milliseconds(self.config.button_press_duration_ms),
            move || {
                tx_command.send(JoystickThreadCommand::ReleaseButton(btn)).unwrap();
            });
        
        self.button_release_guards.insert(btn, guard);
    }
}

impl Drop for VirtualJoystick {
    fn drop(&mut self) {
        self.tx_command.send(JoystickThreadCommand::Terminate).unwrap();
        if let Some(h) = self.handle.take() {
            h.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::drop;

    #[test]
    fn can_create() {
        let vj = VirtualJoystick::new(1, None).unwrap();
        drop(vj);

        thread::sleep(Duration::from_secs(1));
    }

    #[test]
    fn can_press() {
        let mut vj = VirtualJoystick::new(1, Some(VirtualJoystickConfig {
            button_press_duration_ms: 500
        })).unwrap();
        
        vj.press_button(1);
        thread::sleep(Duration::from_secs(1));
        drop(vj);
        thread::sleep(Duration::from_secs(1)); 
    }
}

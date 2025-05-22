use device_query::{DeviceState, DeviceQuery, Keycode};
use std::time::{Duration, Instant};
use std::collections::HashMap;


#[derive(Eq, Hash, PartialEq)]
pub enum InputData {
    Left,
    Right
}

use InputData::*;
impl InputData{
    pub const VALUES: [Self; 2] = [Left, Right];

    #[allow(unused)]
    fn from_keycode(keycode: Keycode) -> Option<InputData>{
        use Keycode::*;
        match keycode {
            Left => Some(InputData::Left),
            Right => Some(InputData::Right),

            _ => None
        }
    }

    fn to_keycode(&self) -> Keycode{
        use Keycode::*;
        match self {
            InputData::Left => Left,
            InputData::Right => Right
        }
    }
}

#[derive(Clone)]
pub struct InputPackage{
    pub move_x: i16,
}

impl InputPackage{
    pub fn new() -> InputPackage{
        InputPackage { move_x: 0 }
    }
}

const COOLDOWN: Duration = Duration::from_millis(5);
//use Keycode::*;
//const KEYS: [Keycode; 10] = [Left, Right, Up, Space, C, LShift, RShift, Z, LControl, RControl];

use std::sync::{Arc, Mutex};
pub fn activate(package_access: Arc<Mutex<InputPackage>>){
    let device_state = DeviceState::new();

    let mut last_pressed: HashMap<Keycode, Instant> = HashMap::new();
    let now = Instant::now();
    for data in InputData::VALUES{
        last_pressed.insert(data.to_keycode(), now);
    }

    loop {
        let keys = DeviceState::get_keys(&device_state);
        let now = Instant::now();

        //pick up inputs from last package
        //this is in curlies so the lock is dropped early
        let mut new_package = {package_access.lock().unwrap().clone()};

        //this is more readable and easier to work with than iter.filter because of the nature of the checks
        //and ownership issues that would force me to insert the new time in the filter closure
        for key in keys{
            if !last_pressed.contains_key(&key){
                continue;
            }

            let since = now.duration_since(last_pressed[&key]);
            last_pressed.insert(key, now);
            if since < COOLDOWN{
                continue;
            }

            use Keycode::*;
            //here it should probably match against a desired behavior instead of key, dont have time for that now
            match key{
                Left => new_package.move_x = -1,
                Right => new_package.move_x = 1,

                _ => panic!("should not have reached the input matching expression with an unregistered key")
            }
        }

        let mut package = package_access.lock().unwrap();
        *package = new_package;

        //tx.send(new_package.clone()).expect("should have been able to send cloned package");

        //the old closure, doesn't do exactly what it's supposed to
        /*for key in keys.iter().filter(|kc| { 
                        let res = last_pressed.contains_key(kc) && now.duration_since(last_pressed[kc]) > COOLDOWN; 
                        if res { last_pressed.insert(**kc, now); } 
                        res })
        }*/
    }
}
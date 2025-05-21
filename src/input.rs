use std::sync::mpsc::Sender;
use device_query::{DeviceState, DeviceQuery, Keycode};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use InputData::*;

#[derive(Eq, Hash, PartialEq)]
pub enum InputData {
    Left,
    Right
}

impl InputData{
    pub const VALUES: [Self; 2] = [Left, Right];

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

const COOLDOWN: Duration = Duration::from_millis(5);

pub fn activate(tx: Sender<InputPackage>){
    let device_state = DeviceState::new();

    let mut last_pressed: HashMap<Keycode, Instant> = HashMap::new();
    let now = Instant::now();
    for data in InputData::VALUES{
        last_pressed.insert(data.to_keycode(), now);
    }

    loop {
        let keys = DeviceState::get_keys(&device_state);
        let now = Instant::now();
        let mut package = InputPackage{move_x: 0};

        for key in keys.iter().filter(|kc| { 
                        let res = last_pressed.contains_key(kc) && now.duration_since(last_pressed[kc]) > COOLDOWN; 
                        if res { last_pressed.insert(**kc, now); } 
                        res })
            {

            //unwrap is safe here because of filter
            match InputData::from_keycode(*key).unwrap(){
                Left => package.move_x = -1,
                Right => package.move_x = 1
            }


            tx.send(package.clone()).expect("should have been able to send cloned package");
        }
    }
}
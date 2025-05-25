use device_query::{DeviceState, DeviceQuery, Keycode};
use std::time::{Duration, Instant};
use std::collections::HashMap;



#[derive(Clone)]
pub struct InputPackage{
    pub move_x: i16,
    pub rotate: Rotation,
    pub hard_drop: bool,
    pub soft_drop: bool,
    pub store: bool,
}

impl InputPackage{
    pub fn new() -> InputPackage{
        InputPackage 
        {
            move_x: 0,
            rotate: Rotation::Not,
            hard_drop: false,
            soft_drop: false,
            store: false,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Rotation {
    Clockwise,
    Counterclockwise,
    Not
}

const COOLDOWN: Duration = Duration::from_millis(5);

use std::sync::{Arc, Mutex};
pub fn activate(package_access: Arc<Mutex<InputPackage>>){
    let device_state = DeviceState::new();

    let mut last_pressed: HashMap<Keycode, Instant> = HashMap::new();
    let now = Instant::now();

    use Keycode::*;
    for key in [Left, Right, Up, Z, LControl, Space, C, LShift, RShift]{
        last_pressed.insert(key, now);
    }

    loop {
        //sleep a little to save the planet, no keyboard should poll fast enough for this to matter
        std::thread::sleep(Duration::from_millis(1));

        let keys = DeviceState::get_keys(&device_state);
        let now = Instant::now();

        //pick up inputs from last package
        //this is in curlies so the lock is dropped early
        let mut new_package = {package_access.lock().unwrap().clone()};

        //this is more readable and easier to work with than iter.filter because of the nature of the checks
        //and ownership issues that would force me to insert the new time in the filter closure
        for key in keys{

            //this is held, not pressed
            if key == Down{
                new_package.soft_drop = true;
                continue;
            }

            if !last_pressed.contains_key(&key){
                continue;
            }

            let since = now.duration_since(last_pressed[&key]);
            last_pressed.insert(key, now);
            if since < COOLDOWN{
                continue;
            }

            
            //here it should probably match against a desired behavior instead of key, dont have time for that now
            match key{
                Left => new_package.move_x = -1,
                Right => new_package.move_x = 1,
                Up => new_package.rotate = Rotation::Clockwise,
                Z | LControl => new_package.rotate = Rotation::Counterclockwise,
                Space => new_package.hard_drop = true,
                C | LShift | RShift => new_package.store = true,

                _ => panic!("should not have reached the input matching expression with an unregistered key")
            }
        }

        if new_package.hard_drop{
            new_package.move_x = 0;
        }

        let mut package = package_access.lock().unwrap();
        *package = new_package;
    }
}
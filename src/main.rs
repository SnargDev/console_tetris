fn main() {
    println!("Hello, world!");

    let test = Arr2D::<i32>::new(4, 4);
    println!("{test:?}");

    println!("arr has {} items", test.arr.iter().count());
    for item in test.arr{
        println!("item: {item}");
    }

    //test.arr.iter().inspect(|item| print!("{item}")).collect::<Vec<i32>>();
}

#[derive(Debug)]
struct Arr2D<T: Copy + Sized + Default>{
    size_x: u8,
    //i don't like the fact this is a vec, i only really need constant size
    //typical arrays can only be declared with compile time constants because they're stored on the heap?
    arr: Vec<T>
}


impl <T: Copy + Sized + Default> Arr2D<T>{

    pub fn get(&self, x: u8, y: u8) -> T{
        self.arr[(self.size_x * y + x) as usize]
    }

    pub fn set(&mut self, x: u8, y:u8, val: T){
        self.arr[(self.size_x * y + x) as usize] = val
    }

    pub fn new(size_x: u8, size_y: u8) -> Self{
        Arr2D{size_x, arr: Vec::with_capacity((size_x*size_y) as usize)}//[size_x * size_y]}
    }
}
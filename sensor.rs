use rand::Rng;

pub fn read_temperature() -> f32 {
       
        let mut rng = rand::thread_rng();
            20.0 + rng.gen_range(0.0..10.0)
}

pub fn read_humidity() -> f32 {
       
        let mut rng = rand::thread_rng();
            40.0 + rng.gen_range(0.0..30.0)
}

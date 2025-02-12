use rand::Rng;

pub fn get_random<T: Ord + rand::distributions::uniform::SampleUniform>(min: T, max: T) -> T {
    rand::thread_rng().gen_range(min..=max)
}
pub fn flip_coin() -> bool {
    rand::random()
}

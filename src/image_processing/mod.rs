use lazy_static::lazy_static;

lazy_static! {
    pub static ref DEFAULT_IMAGE_SIZES: Vec<(u32, u32)> = vec![
        (800, 600),
        (1024, 768),
        (1280, 720),
        (1920, 1080),
    ];
}
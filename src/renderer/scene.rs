use super::camera::Camera;

pub struct Scene {
    camera : Camera,
    
}

impl Scene {
    fn new(aspect_ratio : f32) -> Self {
        Scene {
            camera: Camera::new(aspect_ratio)
        }
    }
}
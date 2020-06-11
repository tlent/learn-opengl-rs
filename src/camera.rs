use nalgebra_glm as glm;

const INITIAL_FOV: f32 = 45.0;
const SPEED: f32 = 2.5;
const SENSITIVITY: f32 = 0.05;

pub struct Camera {
    position: glm::Vec3,
    front: glm::Vec3,
    right: glm::Vec3,
    up: glm::Vec3,
    world_up: glm::Vec3,
    yaw: f32,
    pitch: f32,
    fov: f32,
}

impl Camera {
    pub fn new(position: glm::Vec3, world_up: glm::Vec3, yaw: f32, pitch: f32) -> Self {
        let front = glm::normalize(&glm::vec3(
            yaw.to_radians().cos() * pitch.to_radians().cos(),
            pitch.to_radians().sin(),
            yaw.to_radians().sin() * pitch.to_radians().cos(),
        ));
        let right = glm::normalize(&glm::cross(&front, &world_up));
        let up = glm::normalize(&glm::cross(&right, &front));
        Self {
            position,
            front,
            right,
            up,
            world_up,
            yaw,
            pitch,
            fov: INITIAL_FOV,
        }
    }

    pub fn view_matrix(&self) -> glm::Mat4 {
        glm::look_at(&self.position, &(self.position + self.front), &self.up)
    }

    pub fn move_(&mut self, directions: &[CameraMotion], delta_time: f32) {
        let mut velocity = glm::vec3(0.0, 0.0, 0.0);
        for d in directions {
            match d {
                CameraMotion::Forward => velocity += self.front,
                CameraMotion::Backward => velocity -= self.front,
                CameraMotion::Right => velocity += self.right,
                CameraMotion::Left => velocity -= self.right,
                CameraMotion::Up => velocity += self.up,
                CameraMotion::Down => velocity -= self.up,
            }
        }
        if velocity != glm::vec3(0.0, 0.0, 0.0) {
            self.position += SPEED * velocity.normalize() * delta_time;
        }
    }

    pub fn look(&mut self, mouse_delta: (f32, f32)) {
        let (dx, dy) = mouse_delta;
        self.yaw += SENSITIVITY * dx;
        self.pitch += SENSITIVITY * dy;
        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }
        self.front = glm::normalize(&glm::vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        ));
        self.right = glm::normalize(&glm::cross(&self.front, &self.world_up));
        self.up = glm::normalize(&glm::cross(&self.right, &self.front));
    }

    pub fn zoom(&mut self, scroll_delta: f32) {
        self.fov += scroll_delta;
        if self.fov > 45.0 {
            self.fov = 45.0;
        }
        if self.fov < 1.0 {
            self.fov = 1.0;
        }
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }
}

pub enum CameraMotion {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

use cgmath::{Rotation, Rotation3};
use winit::keyboard::KeyCode;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // Build the view matrix.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        // Build the projection matrix.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // Return the combined matrix.
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_left_rotate_pressed: bool,
    is_right_rotate_pressed: bool,
    is_up_rotate_pressed: bool,
    is_down_rotate_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_left_rotate_pressed: false,
            is_right_rotate_pressed: false,
            is_up_rotate_pressed: false,
            is_down_rotate_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyS => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyD => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::ArrowLeft => {
                self.is_left_rotate_pressed = is_pressed;
                true
            }
            KeyCode::ArrowRight => {
                self.is_right_rotate_pressed = is_pressed;
                true
            }
            KeyCode::ArrowUp => {
                self.is_up_rotate_pressed = is_pressed;
                true
            }
            KeyCode::ArrowDown => {
                self.is_down_rotate_pressed = is_pressed;
                true
            }
            KeyCode::KeyE => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::KeyQ => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;

        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        let speed_limit = 0.05;

        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed * speed_limit;
            camera.target += forward_norm * self.speed * speed_limit;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed * speed_limit;
            camera.target -= forward_norm * self.speed * speed_limit;
        }

        if self.is_left_pressed {
            let right = forward_norm.cross(camera.up);
            camera.eye -= right * self.speed * speed_limit;
            camera.target -= right * self.speed * speed_limit;
        }
        if self.is_right_pressed {
            let right = forward_norm.cross(camera.up);
            camera.eye += right * self.speed * speed_limit;
            camera.target += right * self.speed * speed_limit;
        }

        if self.is_left_rotate_pressed {
            let rotation = cgmath::Quaternion::from_axis_angle(camera.up, cgmath::Deg(self.speed));
            let new_forward = rotation.rotate_vector(forward);
            camera.target = camera.eye + new_forward;
        }
        if self.is_right_rotate_pressed {
            let rotation = cgmath::Quaternion::from_axis_angle(camera.up, cgmath::Deg(-self.speed));
            let new_forward = rotation.rotate_vector(forward);
            camera.target = camera.eye + new_forward;
        }

        let right = forward_norm.cross(camera.up);

        if self.is_up_rotate_pressed {
            let rotation = cgmath::Quaternion::from_axis_angle(right, cgmath::Deg(self.speed));
            let new_forward = rotation.rotate_vector(forward);
            camera.target = camera.eye + new_forward;
        }
        if self.is_down_rotate_pressed {
            let rotation = cgmath::Quaternion::from_axis_angle(right, cgmath::Deg(-self.speed));
            let new_forward = rotation.rotate_vector(forward);
            camera.target = camera.eye + new_forward;
        }

        if self.is_up_pressed {
            camera.eye += camera.up * self.speed * speed_limit;
            camera.target += camera.up * self.speed * speed_limit;
        }
        if self.is_down_pressed {
            camera.eye -= camera.up * self.speed * speed_limit;
            camera.target -= camera.up * self.speed * speed_limit;
        }
    }
}

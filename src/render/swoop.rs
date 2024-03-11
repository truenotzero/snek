use std::mem::{offset_of, size_of};

use crate::{
    common::{as_bytes, AsBytes},
    gl::{self, ArrayBuffer, DrawContext, Shader, Vao},
    math::{Mat4, Vec2, Vec3, Vec4},
    resources,
};

use super::VaoHelper;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

as_bytes!(Vertex);

#[derive(Debug)]
pub struct Swoop {
    vertices: [Vertex; 6],
}

impl Default for Swoop {
    fn default() -> Swoop {
        let corners = [
            Vertex {
                pos: Vec2::new(-0.5, 0.5),
                uv: Vec2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vec2::new(-0.5, -0.5),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vec2::new(0.5, 0.5),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                pos: Vec2::new(0.5, -0.5),
                uv: Vec2::new(1.0, 0.0),
            },
        ];

        Self {
            vertices: [
                corners[0], corners[1], corners[2], corners[3], corners[2], corners[1],
            ],
        }
    }
}

impl Swoop {
    pub fn new<D: Into<Vec2>>(pos: Vec2, scale: f32, direction: D) -> Self {
        Self::default()
            .transform(Mat4::rotate(Into::<Vec2>::into(direction).angle()))
            .transform(Mat4::scale(scale.into()))
            .transform(Mat4::translate((pos, 0.0).into()))
    }

    pub fn transform(mut self, t: Mat4) -> Self {
        for v in &mut self.vertices {
            v.pos = t * v.pos;
        }

        self
    }
}

pub struct SwoopManager<'a> {
    vao: Vao<'a>,
    vbo: ArrayBuffer<'a>,
    shader: Shader<'a>,

    num_swoops: usize,
    max_swoops: usize,
}

impl<'a> SwoopManager<'a> {
    pub fn new(ctx: &'a DrawContext, max_swoops: usize) -> Self {
        let max_swoops = max_swoops * size_of::<Swoop>();
        let vbo = ArrayBuffer::new(ctx);
        vbo.reserve(max_swoops, gl::buffer_flags::DYNAMIC_STORAGE);

        let vao = VaoHelper::new(ctx)
            .bind_buffer(&vbo)
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Vertex>(),
                offset_of!(Vertex, pos),
            )
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Vertex>(),
                offset_of!(Vertex, uv),
            )
            .build();

        Self {
            vao,
            vbo,
            shader: Shader::from_resource(ctx, resources::shaders::SWOOP)
                .expect("bad swoop shader"),

            num_swoops: 0,
            max_swoops,
        }
    }

    pub fn push(&mut self, swoop: Swoop) {
        for v in swoop.vertices {
            if self.num_swoops == self.max_swoops {
                panic!("max swoops")
            }

            self.vbo
                .update(self.num_swoops * size_of::<Vertex>(), unsafe {
                    v.as_bytes()
                });

            self.num_swoops += 1;
        }
    }

    pub fn draw(&mut self) {
        self.vao.apply();
        self.shader.apply();
        gl::call!(DrawArrays(TRIANGLES, 0, self.num_swoops as _));

        self.num_swoops = 0;
    }
}
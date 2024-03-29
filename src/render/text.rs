use std::{
    collections::HashMap,
    mem::{offset_of, size_of},
};

use crate::{
    common::{as_bytes, AsBytes, Error, Result},
    gl::{self, ArrayBuffer, DrawContext, Shader, Texture2D, Uniform, Vao},
    math::{Mat4, Vec2, Vec3},
    resources::{self, Texture},
};

use super::VaoHelper;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

as_bytes!(Vertex);

#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TextNames {
    // tutorial
    Snek,
    SnekGlitch,
    Controls,
    Fruit,
    FruitGlitch,
    FruitGlitchVariant,
    Attack,
    AttackGlitch,
    Empower,
    EmpowerGlitch,
    Enemy,
    EnemyGlitch,
    Shield,
    ShieldGlitch,
    // gameplay
    LuckyGlitch,
    SwarmGlitch,
    BossGlitch,

    _NumTexts,
}

impl TryFrom<u8> for TextNames {
    type Error = Error;

    fn try_from(value: u8) -> Result<TextNames> {
        use TextNames as T;
        Ok(match value {
            0 => T::Snek,
            1 => T::SnekGlitch,
            2 => T::Controls,
            3 => T::Fruit,
            4 => T::FruitGlitch,
            5 => T::FruitGlitchVariant,
            6 => T::Attack,
            7 => T::AttackGlitch,
            8 => T::Empower,
            9 => T::EmpowerGlitch,
            10 => T::Enemy,
            11 => T::EnemyGlitch,
            12 => T::Shield,
            13 => T::ShieldGlitch,

            14 => T::LuckyGlitch,
            15 => T::SwarmGlitch,
            16 => T::BossGlitch,

            _ => Err(Error::InvalidTextNameId)?,
        })
    }
}

impl TextNames {
    fn resource(self) -> Texture {
        use crate::resources::textures::text::*;
        match self {
            Self::Snek => SNEK,
            Self::SnekGlitch => SNEK_GLITCH,
            Self::Controls => CONTROLS,
            Self::Fruit => FRUIT,
            Self::FruitGlitch => FRUIT_GLITCH,
            Self::FruitGlitchVariant => FRUIT_GLITCH_VARIANT,
            Self::Attack => ATTACK,
            Self::AttackGlitch => ATTACK_GLITCH,
            Self::Empower => EMPOWER,
            Self::EmpowerGlitch => EMPOWER_GLITCH,
            Self::Enemy => ENEMY,
            Self::EnemyGlitch => ENEMY_GLITCH,
            Self::Shield => SHIELD,
            Self::ShieldGlitch => SHIELD_GLITCH,

            Self::LuckyGlitch => LUCKY_GLITCH,
            Self::SwarmGlitch => SWARM_GLITCH,
            Self::BossGlitch => BOSS_GLITCH,

            TextNames::_NumTexts => panic!(),
        }
    }

    pub fn dimensions(self) -> Vec2 {
        match self {
            Self::Snek => Vec2::new(62.0, 14.0),
            Self::SnekGlitch => Vec2::new(14.0, 96.0),
            Self::Controls => Vec2::new(142.0, 38.0),
            Self::Fruit => Vec2::new(126.0, 14.0),
            Self::FruitGlitch => Vec2::new(142.0, 192.0),
            Self::FruitGlitchVariant => Vec2::new(142.0, 192.0),
            Self::Attack => Vec2::new(238.0, 14.0),
            Self::AttackGlitch => Vec2::new(174.0, 192.0),
            Self::Empower => Vec2::new(174.0, 14.0),
            Self::EmpowerGlitch => Vec2::new(206.0, 192.0),
            Self::Enemy => Vec2::new(174.0, 14.0),
            Self::EnemyGlitch => Vec2::new(158.0, 192.0),
            Self::Shield => Vec2::new(398.0, 14.0),
            Self::ShieldGlitch => Vec2::new(142.0, 192.0),

            Self::LuckyGlitch => Vec2::new(174.0, 192.0),
            Self::SwarmGlitch => Vec2::new(302.0, 264.0),
            Self::BossGlitch => Vec2::new(126.0, 192.0),

            Self::_NumTexts => panic!(),
        }
    }

    pub fn frames(self) -> usize {
        match self {
            Self::SnekGlitch => 4,
            Self::FruitGlitch => 8,
            Self::FruitGlitchVariant => 8,
            Self::AttackGlitch => 8,
            Self::EmpowerGlitch => 8,
            Self::EnemyGlitch => 8,
            Self::ShieldGlitch => 8,

            Self::LuckyGlitch => 8,
            Self::SwarmGlitch => 11,
            Self::BossGlitch => 8,

            Self::_NumTexts => panic!(),
            _ => 1,
        }
    }
}

const VERTICES_PER_SHAPE: usize = 6;

pub const LETTER_SIZE: f32 = 14.0;
pub const LETTER_GAP_WIDTH: f32 = 2.0;
pub const LINE_SEPARATOR_HEIGHT: f32 = 10.0;

#[derive(Debug)]
pub struct Text {
    name: TextNames,
    frame: usize,
    vertices: [Vertex; VERTICES_PER_SHAPE],
}

impl Text {
    fn new(name: TextNames, frame: usize) -> Self {
        let corners = [
            Vertex {
                pos: Vec2::new(-0.5, 0.5),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vec2::new(-0.5, -0.5),
                uv: Vec2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vec2::new(0.5, 0.5),
                uv: Vec2::new(1.0, 0.0),
            },
            Vertex {
                pos: Vec2::new(0.5, -0.5),
                uv: Vec2::new(1.0, 1.0),
            },
        ];

        Self {
            name,
            frame,
            vertices: [
                corners[0], corners[1], corners[2], corners[3], corners[2], corners[1],
            ],
        }
    }

    pub fn place_at(
        name: TextNames,
        position: Vec2,
        dimensions: Vec2,
        scale: f32,
        frame: usize,
    ) -> Self {
        let frames = name.frames();
        let adjust = if frames > 1 {
            let frame_adjust = 1.0 / frames as f32;
            let whitespace_adjust = LINE_SEPARATOR_HEIGHT / LETTER_SIZE;

            Mat4::scale(Vec2::new(1.0, frame_adjust * whitespace_adjust))
                * Mat4::translate(Vec3::new(LETTER_GAP_WIDTH, -LINE_SEPARATOR_HEIGHT, 0.0))
            // Mat4::default()
        } else {
            Mat4::default()
        };

        let out = Self::new(name, frame)
            .transform(Mat4::scale(dimensions))
            .transform(adjust)
            .transform(Mat4::scale(scale.into()))
            .transform(Mat4::translate((position, 0.0).into()));

        out
    }

    fn transform(mut self, t: Mat4) -> Self {
        for v in &mut self.vertices {
            v.pos = t * v.pos;
        }

        self
    }
}

pub struct TextManager<'a> {
    vao: Vao<'a>,
    vbo: ArrayBuffer<'a>,
    shader: Shader<'a>,

    textures: HashMap<TextNames, Texture2D<'a>>,

    texts: Vec<Text>,
}

impl<'a> TextManager<'a> {
    pub fn new(ctx: &'a DrawContext) -> Self {
        let vbo = ArrayBuffer::new(ctx);
        vbo.reserve(
            size_of::<Vertex>() * VERTICES_PER_SHAPE,
            gl::buffer_flags::DYNAMIC_STORAGE,
        );

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
            shader: Shader::from_resource(ctx, resources::shaders::TEXT).expect("bad text shader"),

            textures: Self::load_textures(ctx),

            texts: Default::default(),
        }
    }

    fn load_textures(ctx: &'a DrawContext) -> HashMap<TextNames, Texture2D<'a>> {
        let mut ret = HashMap::new();
        for text_name_id in 0..(TextNames::_NumTexts as u8) {
            // don't forget to add new text names to the conversion table in try_from
            let text_name = TextNames::try_from(text_name_id).unwrap();

            let image = image::load_from_memory(text_name.resource()).unwrap();
            let image = image.flipv();

            let texture = Texture2D::new(ctx);
            let width = text_name.dimensions().x as _;
            let height = text_name.dimensions().y as _;
            texture.apply();
            // effectively clamp to a transparent background
            gl::call!(TexParameteri(
                texture.type_(),
                TEXTURE_WRAP_S,
                CLAMP_TO_BORDER as _
            ));
            gl::call!(TexParameteri(
                texture.type_(),
                TEXTURE_WRAP_T,
                CLAMP_TO_BORDER as _
            ));
            gl::call!(TexParameteri(
                texture.type_(),
                TEXTURE_MIN_FILTER,
                LINEAR as _
            ));
            gl::call!(TexParameteri(
                texture.type_(),
                TEXTURE_MAG_FILTER,
                LINEAR as _
            ));
            gl::call!(TexImage2D(
                texture.type_(),
                0,
                RGBA as _,
                width,
                height,
                0,
                RGBA,
                UNSIGNED_BYTE,
                image.as_bytes().as_ptr().cast()
            ));

            // push to hashmap
            ret.insert(text_name, texture);
        }

        ret
    }

    pub fn push(&mut self, text: Text) {
        self.texts.push(text);
    }

    const BINDING_TEXT: usize = 0;
    const UNIFORM_CURRENT_FRAME: i32 = 0;
    const UNIFORM_TOTAL_FRAMES: i32 = 1;

    pub fn draw(&mut self) {
        self.vao.apply();
        self.shader.apply();

        for text in &self.texts {
            for (i, v) in text.vertices.iter().enumerate() {
                let bytes = unsafe { v.as_bytes() };
                self.vbo.update(i * size_of::<Vertex>(), bytes);
            }

            self.textures[&text.name].bind(Self::BINDING_TEXT);
            (text.frame as f32).uniform(Self::UNIFORM_CURRENT_FRAME);
            (text.name.frames() as f32).uniform(Self::UNIFORM_TOTAL_FRAMES);
            gl::call!(DrawArrays(TRIANGLES, 0, VERTICES_PER_SHAPE as _));
        }

        self.texts.clear();
    }
}

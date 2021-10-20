use pr47_codegen::pr47_function_bind;
use xjbutil::void::Void;

use crate::data::traits::StaticBase;

pub struct Vec2(f64, f64);
pub struct Vec3(f64, f64, f64);
pub struct Vec4(f64, f64, f64, f64);

impl StaticBase<Vec2> for Void {
    fn type_name() -> String { "std.math.Vec2".to_string() }
}

impl StaticBase<Vec3> for Void {
    fn type_name() -> String { "std.math.Vec3".to_string() }
}

impl StaticBase<Vec4> for Void {
    fn type_name() -> String { "std.math.Vec4".to_string() }
}

#[pr47_function_bind(local)]
pub fn vec2_new(x: f64, y: f64) -> Vec2 { Vec2(x, y) }

#[pr47_function_bind(local)]
pub fn vec3_new(x: f64, y: f64, z: f64) -> Vec3 { Vec3(x, y, z) }

#[pr47_function_bind(local)]
pub fn vec4_new(x: f64, y: f64, z: f64, t: f64) -> Vec4 { Vec4(x, y, z, t) }

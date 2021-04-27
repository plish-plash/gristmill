
pub type Color = palette::LinSrgba;

pub fn black() -> Color {
    Color::new(0., 0., 0., 1.)
}
pub fn white() -> Color {
    Color::new(1., 1., 1., 1.)
}

pub fn encode_color(color: Color) -> [f32; 4] {
    let nonlinear = palette::Srgba::from_linear(color);
    let (r, g, b, a) = nonlinear.into_components();
    [r, g, b, a]
}

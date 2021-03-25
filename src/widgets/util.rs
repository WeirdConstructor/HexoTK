use super::*;

pub fn rect_border(p: &mut dyn Painter,
    border_w: f64, border_clr: (f64, f64, f64),
    fill_clr: (f64, f64, f64), pos: Rect) -> Rect
{
    p.rect_fill(border_clr, pos.x, pos.y, pos.w, pos.h);
    let pos = pos.shrink(border_w, border_w);
    p.rect_fill(fill_clr, pos.x, pos.y, pos.w, pos.h);
    pos
}


pub fn draw_pointer(p: &mut dyn Painter, size: f64, clr: (f64, f64, f64), pos: Rect) {
    p.path_fill(
        clr,
        &mut [
            (pos.x - size,  pos.y - size * 0.5),
            (pos.x + size,  pos.y - size * 0.5),
            (pos.x,         pos.y + size * 0.5),
        ].iter().copied(),
        true);
}

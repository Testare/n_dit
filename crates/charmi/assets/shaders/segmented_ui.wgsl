#import charmi::{
    view,
    view_transform,
    transform,
    globals,
    view_pos,
    TransformCh,
};
#import charmi::box::{
    LINES,
    NORTH,
    EAST,
    SOUTH,
    WEST,
    NORTHSOUTH,
    EASTWEST,
};


@group(2) @binding(0) var<uniform> pt1: vec2<u32>;
@group(2) @binding(1) var<uniform> pt2: vec2<u32>;
@group(2) @binding(2) var<uniform> pt3: vec2<u32>;
@group(2) @binding(3) var<uniform> pt4: vec2<u32>;

// These to be moved to some common Charmi library


fn flash_box(i:u32, t_wrap: u32, timing: u32) {
    if t_wrap <= timing {
        view.cells[i].ch = 0x20u;
        view.cells[i].bg = 0x00u;
    } else if t_wrap <= timing + 20 {
        view.cells[i].fg = 0x09u;
        view.cells[i].bg = 0x00u;
    } else if t_wrap <= timing + 40 {
        view.cells[i].fg = 0x03u;
        view.cells[i].bg = 0x00u;
    } else if t_wrap <= timing + 60 {
        view.cells[i].fg = 0x05u;
        view.cells[i].bg = 0x00u;
    }
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let vid = view_pos(global_id);
    let i = vid.z;
    let x = vid.x;
    let y = vid.y;
    let t = globals.frame_count;
    let widthMinusOne = view.width - 1;
    let heightMinusOne = view.height - 1;

    // Changeable configurations, perhaps make as overrides?
    // TODO use a time based on seconds not frame count so the timing is more dependable
    // TODO Make everything opaque and use default fg/bg while boxes are being drawn, then flash fg and bg a bit before settling. Be careful not to make it an epilipsy trap
    let loopTime = 800u;
    let t_wrap = t % loopTime;

    let boxAStartOffset = 30u;
    let boxAFinishTime  = 250u;
    let boxADiameter = widthMinusOne*2 + (pt3.y - pt1.y)*2+1;
    let progressBoxA = ((t_wrap - min(boxAStartOffset, t_wrap))*boxADiameter)/(boxAFinishTime - boxAStartOffset);

    let boxBStartOffset = 0u;
    let boxBFinishTime  = 200u;
    let boxBDiameter = heightMinusOne*2 + (pt3.x - pt1.x)*2+1;
    let progressBoxB = ((t_wrap - min(boxBStartOffset, t_wrap))*boxBDiameter)/(boxBFinishTime - boxBStartOffset);

    let boxCStartOffset = 60u;
    let boxCFinishTime  = 280u;
    let boxCDiameter = (pt4.x - pt2.x)*2 + (pt4.y - pt2.y)*2+1;
    let progressBoxC = ((t_wrap - min(boxCStartOffset, t_wrap))*boxCDiameter)/(boxCFinishTime - boxCStartOffset);


    // OPTIMIZATION IDEA: Create a storage buffer cache for all the width and height calculations, have some work groups set up these constants, and then run?

    var lineType = 0u;

    if x == 0 {
        // BOX A LEFT
        if y >= pt1.y && y <= pt3.y && y - pt1.y < progressBoxA {
            if y == pt1.y {
               lineType = SOUTH;
            } else if y == pt3.y {
               lineType = NORTH;
            } else {
               lineType = NORTHSOUTH;
            }
        }
    } else if x == pt1.x {
        // BOX B LEFT
        if y < progressBoxB {
            if y == 0u {
               lineType = SOUTH;
            } else if y == heightMinusOne {
               lineType = NORTH;
            } else {
               lineType = NORTHSOUTH;
            }
        }
    } else if x == pt2.x {
        // BOX C LEFT
        if y >= pt2.y && y <= pt4.y 
            && y - pt2.y < progressBoxC {
            if y == pt2.y {
               lineType = SOUTH;
            } else if y == pt4.y {
               lineType = NORTH;
            } else {
               lineType = NORTHSOUTH;
            }
        }
    } else if x == pt3.x {
        // BOX B RIGHT
        if (view.height - y) + view.height + (pt3.x - pt1.x) <= progressBoxB + 1 {
            if y == 0u {
               lineType = SOUTH;
            } else if y == heightMinusOne {
               lineType = NORTH;
            } else {
               lineType = NORTHSOUTH;
            }
        }
    } else if x == pt4.x {
        // BOX C RIGHT
        if y >= pt2.y && y <= pt4.y 
            && (pt4.y )*2 + pt4.x - pt2.x -pt2.y - y < progressBoxC {
            if y == pt2.y {
               lineType = SOUTH;
            } else if y == pt4.y {
               lineType = NORTH;
            } else {
               lineType = NORTHSOUTH;
            }
        }
    } else if x == widthMinusOne {
        // BOX A RIGHT
        if y >= pt1.y && y <= pt3.y 
          && pt3.y - y + pt3.y - pt1.y + widthMinusOne < progressBoxA {
            if y == pt1.y {
               lineType = SOUTH;
            } else if y == pt3.y {
               lineType = NORTH;
            } else {
               lineType = NORTHSOUTH;
            }

          }
    }


    if y == 0 {
        // BOX B TOP
        if x >= pt1.x && x <= pt3.x 
            && (pt3.x + view.height)*2 -x <= progressBoxB + 3 {
            if x == pt1.x {
                lineType |= EAST;
            } else if x == pt3.x {
                lineType |= WEST;
            } else {
                lineType |= EASTWEST;
            }
        }
    } else if y == pt1.y {
        // BOX A TOP
        if (pt3.y + widthMinusOne - pt1.y)*2 + 1 <= (progressBoxA + x) {
            if x == 0 {
                lineType |= EAST;
            } else if x == widthMinusOne {
                lineType |= WEST;
            } else {
                lineType |= EASTWEST;
            }
        }
    } else if y == pt2.y {
        // BOX C TOP
        if x >= pt2.x && x <= pt4.x && 
            (pt4.x + pt4.y - pt2.y)*2 - pt2.x - x + 1 <= progressBoxC {
            if x == pt2.x {
                lineType |= EAST;
            } else if x == pt4.x {
                lineType |= WEST;
            } else {
                lineType |= EASTWEST;
            }
        }

    } else if y == pt3.y {
        // BOX A BOTTOM
        if x + pt3.y < progressBoxA + pt1.y {
            if x == 0 {
                lineType |= EAST;
            } else if x == widthMinusOne {
                lineType |= WEST;
            } else {
                lineType |= EASTWEST;
            }
        }
    } else if y == pt4.y {
        // BOX C BOTTOM
        if x >= pt2.x && x <= pt4.x && 
            (x - pt2.x) + pt4.y - pt2.y + 1 <= progressBoxC {
            if x == pt2.x {
                lineType |= EAST;
            } else if x == pt4.x {
                lineType |= WEST;
            } else {
                lineType |= EASTWEST;
            }
        }

    } else if y == heightMinusOne {
        // BOX B BOTTOM
        if x >= pt1.x && x <= pt3.x && (x - pt1.x) + view.height <= progressBoxB {
            if x == pt1.x {
                lineType |= EAST;
            } else if x == pt3.x {
                lineType |= WEST;
            } else {
                lineType |= EASTWEST;
            }
        }
    }

    if lineType != 0 {
        view.cells[i].ch = LINES[lineType];
        view.cells[i].bg = 0x00u;
        view.cells[i].fg = 0x07u;
    } else if t_wrap <= 300u {
        view.cells[i].ch = 0x20u;
        view.cells[i].bg = 0x00u;
    } else if x < pt1.x && y > pt1.y && y < pt3.y {
        flash_box(i, t_wrap, 360u);
    } else if x > pt1.x && x < pt3.x && y < pt1.y {
        flash_box(i, t_wrap, 390u);
    } else if x > pt1.x && x < pt3.x && y > pt1.y && y < pt3.y {
        flash_box(i, t_wrap, 300u);
    } else if x > pt3.x && y > pt1.y && y < pt3.y {
        flash_box(i, t_wrap, 420u);
    } else if x > pt1.x && x < pt3.x && y > pt3.y {
        flash_box(i, t_wrap, 450u);
    } else if x > pt3.x && x < pt4.x && y > pt3.y && y < pt4.y {
        flash_box(i, t_wrap, 490u);
    } else if x > pt3.x && y < pt1.y {
        flash_box(i, t_wrap, 470u);
    } else {
        view.cells[i].ch = 0x20u;
        view.cells[i].bg = 0x00u;
    }
}

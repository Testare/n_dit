#import charmi::{view, CharmiCell};

@group(2) @binding(0) var<storage> sprite: charmi::CharmiImage;

// This only works for opaque images, because handling cracking would require some pre-processing
// or something in order to prevent characters with alpha from being drawn twice in the event of cracking.
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = charmi::sprite_pos(global_id);
    if pos.x < 0 {
        return; // Not part of the sprite
    }
    if any(vec2u(pos) >= vec2(sprite.width, sprite.height)) {
        return;
    }

    /// Get the values for the src (sprite) and dest (view)

    var dest_i = global_id.x;
    var dest_i_right = dest_i + 1;
    var pos_i = u32(pos.y) * sprite.width + u32(pos.x);
    var src_left = sprite.cells[pos_i];
    var src_right: CharmiCell;
    if u32(pos.x) + 1 == sprite.width {
        src_right = charmi::GAP_CELL;
    } else {
        src_right = sprite.cells[pos_i+1];
    }
    var dest_at_beginning = dest_i % view.width == 0;
    var dest_at_end = dest_i_right % view.width == 0;
    var doublewidth = false;

    /// If dest is double-width character, we'll handle both cells.

    if view.cells[dest_i].ch == charmi::SUPPRESSED_CHAR {
      if pos.x == 0 {
        // If this is the first cell in the sprite and it covers the second half of a doublewidth
        // character in the destination, we'll have to manage the cell before the sprite too.
        doublewidth = true;
        src_right = src_left;
        src_left = charmi::GAP_CELL;
        dest_i_right = dest_i;
        dest_i -= 1u;
      } else {
        // This will be handled by another item.
        return;
      }
    } else if !dest_at_end && view.cells[dest_i_right].ch == charmi::SUPPRESSED_CHAR {
      doublewidth = true;
    }

    var dest_left = view.cells[dest_i];
    var dest_right = view.cells[dest_i_right];

    /// If there is only one gap on a double-width character, we need to crack the destination
    let src_left_opaque = src_left.ch != 0;
    let src_right_opaque = src_right.ch != 0;

    if doublewidth && src_left_opaque != src_right_opaque {
        dest_left.ch = dest_right.bg;
        dest_right = CharmiCell(
            dest_right.fg,
            dest_left.fg,
            dest_left.bg,
            dest_left.attr
        );
    }

    /// Crack src if it has a double-width character that won't fit on the destination.

    if dest_at_beginning && src_left.ch == charmi::SUPPRESSED_CHAR  {
        let src_pre_left = sprite.cells[pos_i - 1];
        src_left = CharmiCell(
            src_left.fg,
            src_pre_left.fg,
            src_pre_left.bg,
            src_pre_left.attr
        );
    } else if !doublewidth && dest_at_end && src_right.ch == charmi::SUPPRESSED_CHAR {
        src_left.ch = src_right.bg;
    }

    /// ACTUALLY DO THE DRAWING

    if doublewidth && (src_left_opaque || src_right_opaque) {
        // If we're drawing two gaps onto a double-width character,
        // do not modify the suppresed char.
        //
        // Otherwise, this case applies and we draw down.
        if src_right.ch != 0 {
            dest_right.ch = src_right.ch;
        }
        if src_right.bg < charmi::NO_COLOR {
            dest_right.bg = src_right.bg;
        }
        if src_right.fg < charmi::NO_COLOR {
            dest_right.fg = src_right.fg;
        }
        view.cells[dest_i_right] = dest_right;
    }
    if src_left.ch != 0 {
        dest_left.ch = src_left.ch;
    }
    if src_left.bg < charmi::NO_COLOR {
        dest_left.bg = src_left.bg;
    }
    if src_left.fg < charmi::NO_COLOR {
        dest_left.fg = src_left.fg;
    }
    view.cells[dest_i] = dest_left;
}

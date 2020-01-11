module petri_dish(h, inner_r, thickness) {
    difference() {
        cylinder(h = h, r = inner_r + thickness);
        translate([0, 0, thickness]) {
            cylinder(h = h, r = inner_r);
        }
    }
}
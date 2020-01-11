include <constants.scad>;
include <modules.scad>;

inner_radius = tray_inner_radius - margin - thickness;

module base() {
    petri_dish(drainer_height, inner_radius, thickness);
}

module hole() {
    module base() {
        interval = drainer_hole_size + thickness;
        repeat = 6;
        size = drainer_hole_size * repeat + thickness * (repeat - 1);

        translate([-size / 2, -size / 2, 0]) {
            for (i = [0 : repeat - 1]) {
                for (j = [0 : repeat - 1]) {
                    translate([j * interval, i * interval, 0]) {
                        cube(drainer_hole_size);
                    }
                }
            }
        }
    }

    module mask() {
        translate([0, 0, -5]) {
            cylinder(r = inner_radius - thin, h = drainer_hole_size + 10);
        }
    }

    translate([0, 0, -thin]) {
        intersection() {
            base();
            mask();
        }
    }
}

difference() {
    base();
    hole();
}

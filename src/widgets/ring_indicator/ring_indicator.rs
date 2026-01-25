use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::subclass::widget::WidgetImpl;
// Removed unused imports
use std::cell::Cell;
use std::f64::consts::PI;

#[derive(Debug, Default)]
pub struct RingIndicatorPriv {
    percentage: Cell<f64>,
}


#[glib::object_subclass]
impl ObjectSubclass for RingIndicatorPriv {
    const NAME: &'static str = "RingIndicator";
    type Type = RingIndicator;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("ring-indicator");
    }
}


impl ObjectImpl for RingIndicatorPriv {}

impl WidgetImpl for RingIndicatorPriv {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let percentage = self.percentage.get();

        // Use fixed size for drawing (matches set_size_request)
        let width = self.obj().width() as f64;
        let height = self.obj().height() as f64;
        let center_x = width / 2.0;
        let center_y = height / 2.0;
    let radius = (f64::min(width, height) / 2.0) - 1.0;
        let line_width = 2.0;

    // Draw background ring
    let rect = gtk::graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
    let cr = snapshot.append_cairo(&rect);
        cr.set_line_width(line_width);
        cr.set_source_rgba(0.3, 0.3, 0.3, 0.5);
        cr.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
        let _ = cr.stroke();

        // Draw filled portion
        if percentage > 0.0 {
            let start_angle = -PI / 2.0;
            let end_angle = start_angle + (2.0 * PI * percentage / 100.0);
            if percentage >= 90.0 {
                cr.set_source_rgb(1.0, 0.2, 0.2);
            } else if percentage >= 70.0 {
                cr.set_source_rgb(1.0, 0.8, 0.0);
            } else {
                cr.set_source_rgb(0.8, 0.8, 0.8);
            }
            cr.arc(center_x, center_y, radius, start_angle, end_angle);
            let _ = cr.stroke();
        }

    }
}

glib::wrapper! {
    pub struct RingIndicator(ObjectSubclass<RingIndicatorPriv>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RingIndicator {
    pub fn new() -> Self {
        let obj: Self = glib::Object::new();
            obj.set_size_request(16, 16);
        obj
    }

    pub fn set_percentage(&self, percentage: f64) {
        let imp = self.imp();
        imp.percentage.set(percentage.clamp(0.0, 100.0));
        self.queue_draw();
    }

    pub fn percentage(&self) -> f64 {
        let imp = self.imp();
        imp.percentage.get()
    }

}

impl Default for RingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

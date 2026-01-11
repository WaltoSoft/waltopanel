use gtk::{Widget, glib::object::Cast, prelude::WidgetExt};

use crate::traits::CompositeWidget;
use crate::widgets::{PanelButton, PanelButtonGroup, RingIndicator};
use super::{SystemMetricsService, SystemMetrics};

pub struct SystemMetricsButton {
  button_group: PanelButtonGroup,
  cpu_button: PanelButton,
  memory_button: PanelButton,
  cpu_ring: RingIndicator,
  memory_ring: RingIndicator,
}

impl SystemMetricsButton {
  pub fn new() -> Self {
    let metrics = SystemMetricsService::start();

    // Create ring indicators
    let cpu_ring = RingIndicator::new();
    cpu_ring.set_label("CPU");

    let memory_ring = RingIndicator::new();
    memory_ring.set_label("MEM");

    // Create panel buttons
    let cpu_button = PanelButton::new();
    let memory_button = PanelButton::new();

    // Set the ring indicators as custom widgets in the panel buttons
    cpu_button.set_custom_widget(Some(&cpu_ring.clone().upcast()));
    memory_button.set_custom_widget(Some(&memory_ring.clone().upcast()));

    // Update initial UI
    Self::update_cpu_ui(&cpu_button, &cpu_ring, &metrics);
    Self::update_memory_ui(&memory_button, &memory_ring, &metrics);

    // Create button group and add buttons
    let button_group = PanelButtonGroup::new();
    button_group.add_button(&cpu_button);
    button_group.add_button(&memory_button);

    // Subscribe to metrics updates
    let cpu_button_clone = cpu_button.clone();
    let memory_button_clone = memory_button.clone();
    let cpu_ring_clone = cpu_ring.clone();
    let memory_ring_clone = memory_ring.clone();
    SystemMetricsService::subscribe(move |metrics| {
      Self::update_cpu_ui(&cpu_button_clone, &cpu_ring_clone, &metrics);
      Self::update_memory_ui(&memory_button_clone, &memory_ring_clone, &metrics);
    });

    // Stop service when destroyed
    button_group.connect_destroy(|_| {
      SystemMetricsService::stop();
    });

    Self {
      button_group,
      cpu_button,
      memory_button,
      cpu_ring,
      memory_ring,
    }
  }

  fn update_cpu_ui(panel_button: &PanelButton, ring: &RingIndicator, metrics: &SystemMetrics) {
    // Update ring percentage
    ring.set_percentage(metrics.cpu.overall_usage as f64);

    // Build tooltip with per-core info
    let mut tooltip = format!("CPU: {}%\n", metrics.cpu.overall_usage as u8);
    for (i, usage) in metrics.cpu.per_core_usage.iter().enumerate() {
      tooltip.push_str(&format!("Core {}: {}%\n", i + 1, *usage as u8));
    }

    panel_button.set_tooltip_text(Some(tooltip.trim_end()));
  }

  fn update_memory_ui(panel_button: &PanelButton, ring: &RingIndicator, metrics: &SystemMetrics) {
    // Update ring percentage
    ring.set_percentage(metrics.memory.usage_percentage as f64);

    panel_button.set_tooltip_text(Some(&format!(
      "Memory: {}%\n{} MB / {} MB",
      metrics.memory.usage_percentage as u8,
      metrics.memory.used_mb,
      metrics.memory.total_mb
    )));
  }
}

impl CompositeWidget for SystemMetricsButton {
  fn widget(&self) -> Widget {
    self.button_group.clone().upcast()
  }
}
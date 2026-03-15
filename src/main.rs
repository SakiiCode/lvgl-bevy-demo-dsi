use std::mem::MaybeUninit;

use crate::jd9365::init_lcd;
use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_svc::sys::{
    MALLOC_CAP_DEFAULT, esp_lcd_panel_draw_bitmap, heap_caps_get_info, multi_heap_info_t,
    xTaskGetTickCount,
};
use lv_bevy_ecs::{
    display::{Display, DrawBuffer},
    functions::*,
    sys::{LV_DEF_REFR_PERIOD, LV_NO_TIMER_READY, lv_mem_monitor_t},
    widgets::{Label, Widget},
};

mod jd9365;

#[unsafe(no_mangle)]
pub fn get_memory_stats(monitor: &mut lv_mem_monitor_t) {
    #[allow(static_mut_refs)]
    unsafe {
        static mut MEM_INFO: multi_heap_info_t =
            unsafe { MaybeUninit::<multi_heap_info_t>::zeroed().assume_init() };
        heap_caps_get_info(&mut MEM_INFO, MALLOC_CAP_DEFAULT);
        monitor.free_cnt = MEM_INFO.free_blocks;
        monitor.used_cnt = MEM_INFO.allocated_blocks;
        monitor.free_biggest_size = MEM_INFO.largest_free_block;
        monitor.used_cnt = MEM_INFO.allocated_blocks;
        monitor.max_used = usize::max(MEM_INFO.allocated_blocks, monitor.max_used);
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let panel_handle = init_lcd();

    assert_ne!(panel_handle, core::ptr::null_mut());

    lv_init();

    lv_tick_set_cb(|| unsafe { xTaskGetTickCount() });

    const HOR_RES: u32 = 800;
    const VER_RES: u32 = 1280;
    const LINE_HEIGHT: u32 = VER_RES / 40;

    let mut display = Display::create(HOR_RES as i32, VER_RES as i32);
    let buffer =
        DrawBuffer::<{ (HOR_RES * LINE_HEIGHT) as usize }, Rgb565>::create(HOR_RES, LINE_HEIGHT);
    log::info!("Display OK");
    display.register(buffer, |refresh| {
        let area = refresh.rectangle;

        let x_start = area.top_left.x;
        let y_start = area.top_left.y;
        let x_end = area.bottom_right().unwrap().x + 1;
        let y_end = area.bottom_right().unwrap().y + 1;

        unsafe {
            esp_lcd_panel_draw_bitmap(
                panel_handle,
                x_start,
                y_start,
                x_end,
                y_end,
                refresh.colors as *const _ as *const _,
            );
        }
    });

    log::info!("Draw Buffer OK");

    let mut label = Label::create_widget();
    lv_label_set_text(&mut label, c"asdasd");
    Widget::leak(label);

    let mut tick = unsafe { xTaskGetTickCount() };

    loop {
        unsafe {
            let delay = lv_timer_handler();
            if delay != LV_NO_TIMER_READY && delay > 0 {
                esp_idf_svc::sys::xTaskDelayUntil(&mut tick, delay);
            } else {
                esp_idf_svc::sys::vTaskDelay(LV_DEF_REFR_PERIOD);
            }
        }
    }
}

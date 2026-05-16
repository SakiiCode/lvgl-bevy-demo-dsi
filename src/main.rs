use std::{
    mem::MaybeUninit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_svc::sys::{
    MALLOC_CAP_DEFAULT, MALLOC_CAP_DMA, MALLOC_CAP_SPIRAM, esp_lcd_dpi_panel_event_callbacks_t,
    esp_lcd_dpi_panel_register_event_callbacks, esp_lcd_panel_draw_bitmap, esp_log_level_set,
    esp_log_level_t_ESP_LOG_DEBUG, esp_log_level_t_ESP_LOG_INFO, heap_caps_get_info,
    heap_caps_malloc, multi_heap_info_t, vPortYield, xTaskGetTickCount,
};
use lv_bevy_ecs::{
    display::{Display, RenderMode},
    functions::*,
    sys::{LV_COLOR_DEPTH, LV_DEF_REFR_PERIOD, lv_mem_monitor_t},
    widgets::Label,
};

mod hx8394;
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

    esp_idf_svc::log::EspLogger::initialize_default();

    lv_init();
    lv_bevy_ecs::logging::connect();

    unsafe {
        esp_log_level_set(c"*".as_ptr(), esp_log_level_t_ESP_LOG_DEBUG);
        esp_log_level_set(c"lcd.dsi.dpi".as_ptr(), esp_log_level_t_ESP_LOG_INFO);
    }

    log::info!("Hello, world!");

    // let panel_handle = crate::jd9365::init_lcd();
    let panel_handle = crate::hx8394::init_lcd();

    assert_ne!(panel_handle, core::ptr::null_mut());

    lv_tick_set_cb(|| unsafe { xTaskGetTickCount() });

    const HOR_RES: u32 = 720;
    const VER_RES: u32 = 1280;
    const LINE_HEIGHT: u32 = VER_RES / 40;

    let mut display = Display::new(HOR_RES as i32, VER_RES as i32);
    // let buffer =
    //     DrawBuffer::<{ (HOR_RES * LINE_HEIGHT) as usize }, Rgb565>::new(HOR_RES, LINE_HEIGHT);
    const BUFFER_LEN: usize = (HOR_RES * LINE_HEIGHT * LV_COLOR_DEPTH / 8) as usize;
    let buffer = unsafe {
        let ptr = heap_caps_malloc(BUFFER_LEN, MALLOC_CAP_SPIRAM | MALLOC_CAP_DMA).cast::<u8>();
        core::slice::from_raw_parts_mut(ptr, BUFFER_LEN)
    };
    log::info!("Display OK");

    let dsi_done = Arc::new(AtomicBool::new(false));

    let mut dsi_done_clone = dsi_done.clone();

    unsafe {
        let cbs = esp_lcd_dpi_panel_event_callbacks_t {
            on_color_trans_done: Some(crate::hx8394::notify_lvgl_flush_ready),
            ..Default::default()
        };
        esp_lcd_dpi_panel_register_event_callbacks(
            panel_handle,
            &cbs,
            (&raw mut dsi_done_clone).cast(),
        );
    }

    unsafe {
        display.register_raw::<_, BUFFER_LEN, Rgb565>(buffer, RenderMode::Partial, |refresh| {
            let area = refresh.rectangle;

            let start = area.top_left;
            let end = area.bottom_right().unwrap();

            esp_lcd_panel_draw_bitmap(
                panel_handle,
                start.x,
                start.y,
                end.x + 1,
                end.y + 1,
                refresh.colors as *const _ as *const _,
            );
            while !dsi_done.load(Ordering::Relaxed) {
                vPortYield();
            }
            dsi_done.store(false, Ordering::Relaxed);
        });
    }

    log::info!("Draw Buffer OK");

    let mut label = Label::new();
    label.set_text(c"asdasd");
    label.leak();

    unsafe {
        esp_idf_svc::sys::heap_caps_check_integrity_all(true);
    }

    loop {
        unsafe {
            let mut tick = xTaskGetTickCount();
            let next_timer = lv_timer_handler();
            match next_timer {
                NextTimerPeriod::Ready => {
                    continue;
                }
                NextTimerPeriod::AfterMs(delay) => {
                    esp_idf_svc::sys::xTaskDelayUntil(&mut tick, delay.get());
                }
                NextTimerPeriod::Never => {
                    esp_idf_svc::sys::vTaskDelay(LV_DEF_REFR_PERIOD);
                }
            }
        }
    }
}

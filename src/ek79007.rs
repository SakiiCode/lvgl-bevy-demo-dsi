use std::ffi::c_void;

use esp_idf_svc::{
    hal::lcd::PanelHandle,
    sys::{
        QueueHandle_t, ek79007_vendor_config_t, ek79007_vendor_config_t__bindgen_ty_1,
        esp_lcd_dbi_io_config_t, esp_lcd_dpi_panel_config_t,
        esp_lcd_dpi_panel_config_t_extra_dpi_panel_flags, esp_lcd_dpi_panel_event_data_t,
        esp_lcd_dsi_bus_config_t, esp_lcd_dsi_bus_handle_t, esp_lcd_new_dsi_bus,
        esp_lcd_new_panel_ek79007, esp_lcd_new_panel_io_dbi, esp_lcd_panel_dev_config_t,
        esp_lcd_panel_disp_on_off, esp_lcd_panel_handle_t, esp_lcd_panel_init,
        esp_lcd_panel_io_handle_t, esp_lcd_panel_reset, esp_lcd_video_timing_t,
        esp_ldo_acquire_channel, esp_ldo_channel_config_t, esp_ldo_channel_handle_t,
        lcd_color_rgb_pixel_format_t_LCD_COLOR_PIXEL_FORMAT_RGB565,
        soc_module_clk_t_SOC_MOD_CLK_PLL_F240M, xQueueGiveFromISR,
    },
};

#[allow(unused)]
pub fn init_lcd() -> PanelHandle {
    unsafe {
        log::info!("MIPI DSI PHY Powered on");

        let mut ldo_mipi_phy = esp_ldo_channel_handle_t::default();
        let ldo_mipi_phy_config = esp_ldo_channel_config_t {
            chan_id: 3,
            voltage_mv: 2500,
            ..Default::default()
        };
        esp_ldo_acquire_channel(&ldo_mipi_phy_config, &mut ldo_mipi_phy);

        log::info!("Initialize MIPI DSI bus");
        let mut mipi_dsi_bus = esp_lcd_dsi_bus_handle_t::default();
        let bus_config = esp_lcd_dsi_bus_config_t {
            bus_id: 0,
            num_data_lanes: 2,
            phy_clk_src: 0,
            lane_bit_rate_mbps: 900,
        };
        esp_lcd_new_dsi_bus(&bus_config, &mut mipi_dsi_bus);

        log::info!("Install panel IO");
        let mut mipi_dbi_io = esp_lcd_panel_io_handle_t::default();
        let dbi_config = esp_lcd_dbi_io_config_t {
            virtual_channel: 0,
            lcd_cmd_bits: 8,
            lcd_param_bits: 8,
        };
        esp_lcd_new_panel_io_dbi(mipi_dsi_bus, &dbi_config, &mut mipi_dbi_io);

        log::info!("Install EK79007 panel driver");
        let mut panel_handle = esp_lcd_panel_handle_t::default();
        let mut dpi_config_flags = esp_lcd_dpi_panel_config_t_extra_dpi_panel_flags::default();
        dpi_config_flags.set_use_dma2d(1);
        let dpi_config = esp_lcd_dpi_panel_config_t {
            virtual_channel: 0,
            dpi_clk_src: soc_module_clk_t_SOC_MOD_CLK_PLL_F240M,
            dpi_clock_freq_mhz: 52,
            pixel_format: lcd_color_rgb_pixel_format_t_LCD_COLOR_PIXEL_FORMAT_RGB565,
            num_fbs: 1,
            video_timing: esp_lcd_video_timing_t {
                h_size: 1024,
                v_size: 600,
                hsync_pulse_width: 10,
                hsync_back_porch: 160,
                hsync_front_porch: 160,
                vsync_pulse_width: 1,
                vsync_back_porch: 23,
                vsync_front_porch: 12,
            },
            flags: dpi_config_flags,
            ..Default::default()
        };

        let mut vendor_config = ek79007_vendor_config_t {
            mipi_config: ek79007_vendor_config_t__bindgen_ty_1 {
                lane_num: 2,
                dsi_bus: mipi_dsi_bus,
                dpi_config: &dpi_config,
                ..Default::default()
            },
            ..Default::default()
        };
        //vendor_config.flags.set_use_mipi_interface(1);
        let panel_config = esp_lcd_panel_dev_config_t {
            reset_gpio_num: -1, // Set to -1 if not use
            //rgb_ele_order: lcd_rgb_element_order_t_LCD_RGB_ELEMENT_ORDER_RGB, // Implemented by LCD command `36h`
            bits_per_pixel: 16, // Implemented by LCD command `3Ah` (16/18/24)
            vendor_config: &mut vendor_config as *mut _ as *mut c_void,
            ..Default::default()
        };
        esp_lcd_new_panel_ek79007(mipi_dbi_io, &panel_config, &mut panel_handle);
        esp_lcd_panel_reset(panel_handle);
        esp_lcd_panel_init(panel_handle);
        esp_lcd_panel_disp_on_off(panel_handle, true);

        panel_handle
    }
}

#[allow(unused)]
pub extern "C" fn notify_lvgl_flush_ready(
    _panel: esp_lcd_panel_handle_t,
    _edata: *mut esp_lcd_dpi_panel_event_data_t,
    user_ctx: *mut c_void,
) -> bool {
    unsafe {
        let semaphore: QueueHandle_t = user_ctx.cast();
        let mut ctx_sw_needed = 0i32;
        xQueueGiveFromISR(semaphore, &mut ctx_sw_needed);
    }
    return false;
}

use std::ffi::c_void;

use esp_idf_svc::sys::{
    esp_lcd_dbi_io_config_t, esp_lcd_dpi_panel_config_t,
    esp_lcd_dpi_panel_config_t_extra_dpi_panel_flags, esp_lcd_dsi_bus_config_t,
    esp_lcd_dsi_bus_handle_t, esp_lcd_new_dsi_bus, esp_lcd_new_panel_io_dbi,
    esp_lcd_new_panel_jd9365, esp_lcd_panel_dev_config_t, esp_lcd_panel_disp_on_off,
    esp_lcd_panel_handle_t, esp_lcd_panel_init, esp_lcd_panel_io_handle_t, esp_lcd_panel_reset,
    esp_lcd_video_timing_t, esp_ldo_acquire_channel, esp_ldo_channel_config_t,
    esp_ldo_channel_handle_t, jd9365_vendor_config_t, jd9365_vendor_config_t__bindgen_ty_1,
    lcd_color_format_t_LCD_COLOR_FMT_RGB888, soc_module_clk_t_SOC_MOD_CLK_PLL_F240M,
};

pub fn init_lcd() {
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
            lane_bit_rate_mbps: 1500.0,
        };
        esp_lcd_new_dsi_bus(&bus_config, &mut mipi_dsi_bus);

        log::info!("Install panel IO");
        let mut mipi_dbi_io = esp_lcd_panel_io_handle_t::default();
        let dbi_config = esp_lcd_dbi_io_config_t {
            lcd_cmd_bits: 8,
            lcd_param_bits: 8,
            virtual_channel: 0,
        };
        esp_lcd_new_panel_io_dbi(mipi_dsi_bus, &dbi_config, &mut mipi_dbi_io);

        log::info!("Install JD9365S panel driver");
        let mut panel_handle = esp_lcd_panel_handle_t::default();
        let mut dpi_config_flags = esp_lcd_dpi_panel_config_t_extra_dpi_panel_flags::default();
        dpi_config_flags.set_use_dma2d(1);
        let dpi_config = esp_lcd_dpi_panel_config_t {
            dpi_clk_src: soc_module_clk_t_SOC_MOD_CLK_PLL_F240M,
            dpi_clock_freq_mhz: 80.0,
            virtual_channel: 0,
            pixel_format: lcd_color_format_t_LCD_COLOR_FMT_RGB888,
            num_fbs: 1,
            video_timing: esp_lcd_video_timing_t {
                h_size: 800,
                v_size: 1280,
                hsync_back_porch: 20,
                hsync_pulse_width: 20,
                hsync_front_porch: 40,
                vsync_back_porch: 10,
                vsync_pulse_width: 4,
                vsync_front_porch: 30,
            },
            ..Default::default()
        }; //JD9365_800_1280_PANEL_60HZ_DPI_CONFIG(EXAMPLE_MIPI_DPI_PX_FORMAT);
        let mut vendor_config = jd9365_vendor_config_t {
            mipi_config: jd9365_vendor_config_t__bindgen_ty_1 {
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
            //rgb_ele_order : lcd_color_format_t_LCD_COLOR_FMT_RGB888,     // Implemented by LCD command `36h`
            bits_per_pixel: 24, // Implemented by LCD command `3Ah` (16/18/24)
            vendor_config: &mut vendor_config as *mut _ as *mut c_void,
            ..Default::default()
        };
        (esp_lcd_new_panel_jd9365(mipi_dbi_io, &panel_config, &mut panel_handle));
        (esp_lcd_panel_reset(panel_handle));
        (esp_lcd_panel_init(panel_handle));
        (esp_lcd_panel_disp_on_off(panel_handle, true));
    }
}

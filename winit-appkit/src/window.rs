#![allow(clippy::unnecessary_cast)]

use std::sync::Arc;

use dispatch2::MainThreadBound;
use dpi::{Position, Size};
use objc2::rc::{autoreleasepool, Retained};
use objc2::{define_class, MainThreadMarker, Message};
use objc2_app_kit::{NSPanel, NSResponder, NSWindow};
use objc2_foundation::NSObject;
use winit_core::cursor::Cursor;
use winit_core::error::RequestError;
use winit_core::icon::Icon;
use winit_core::monitor::{Fullscreen, MonitorHandle as CoreMonitorHandle};
use winit_core::window::{
    ImeCapabilities, ImeRequest, ImeRequestError, Theme, UserAttentionType, Window as CoreWindow,
    WindowAttributes, WindowButtons, WindowId, WindowLevel,
};

use super::event_loop::ActiveEventLoop;
use super::window_delegate::WindowDelegate;

#[derive(Debug)]
pub(crate) struct Window {
    window: MainThreadBound<Retained<NSWindow>>,
    /// The window only keeps a weak reference to this, so we must keep it around here.
    delegate: MainThreadBound<Retained<WindowDelegate>>,
}

impl Window {
    pub(crate) fn new(
        window_target: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Result<Self, RequestError> {
        let mtm = window_target.mtm;
        let delegate =
            autoreleasepool(|_| WindowDelegate::new(&window_target.app_state, attributes, mtm))?;
        Ok(Window {
            window: MainThreadBound::new(delegate.window().retain(), mtm),
            delegate: MainThreadBound::new(delegate, mtm),
        })
    }

    pub(crate) fn maybe_wait_on_main<R: Send>(
        &self,
        f: impl FnOnce(&WindowDelegate) -> R + Send,
    ) -> R {
        self.delegate.get_on_main(|delegate| f(delegate))
    }

    #[inline]
    pub(crate) fn raw_window_handle_rwh_06(
        &self,
    ) -> Result<rwh_06::RawWindowHandle, rwh_06::HandleError> {
        if let Some(mtm) = MainThreadMarker::new() {
            Ok(self.delegate.get(mtm).raw_window_handle_rwh_06())
        } else {
            Err(rwh_06::HandleError::Unavailable)
        }
    }

    #[inline]
    pub(crate) fn raw_display_handle_rwh_06(
        &self,
    ) -> Result<rwh_06::RawDisplayHandle, rwh_06::HandleError> {
        Ok(rwh_06::RawDisplayHandle::AppKit(rwh_06::AppKitDisplayHandle::new()))
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // Restore the video mode.
        if matches!(self.fullscreen(), Some(Fullscreen::Exclusive(_, _))) {
            self.set_fullscreen(None);
        }

        self.window.get_on_main(|window| autoreleasepool(|_| window.close()))
    }
}

impl rwh_06::HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<rwh_06::DisplayHandle<'_>, rwh_06::HandleError> {
        let raw = self.raw_display_handle_rwh_06()?;
        unsafe { Ok(rwh_06::DisplayHandle::borrow_raw(raw)) }
    }
}

impl rwh_06::HasWindowHandle for Window {
    fn window_handle(&self) -> Result<rwh_06::WindowHandle<'_>, rwh_06::HandleError> {
        let raw = self.raw_window_handle_rwh_06()?;
        unsafe { Ok(rwh_06::WindowHandle::borrow_raw(raw)) }
    }
}

impl CoreWindow for Window {
    fn id(&self) -> winit_core::window::WindowId {
        self.maybe_wait_on_main(|delegate| delegate.id())
    }

    fn scale_factor(&self) -> f64 {
        self.maybe_wait_on_main(|delegate| delegate.scale_factor())
    }

    fn request_redraw(&self) {
        self.maybe_wait_on_main(|delegate| delegate.request_redraw());
    }

    fn pre_present_notify(&self) {
        self.maybe_wait_on_main(|delegate| delegate.pre_present_notify());
    }

    fn reset_dead_keys(&self) {
        self.maybe_wait_on_main(|delegate| delegate.reset_dead_keys());
    }

    fn surface_position(&self) -> dpi::PhysicalPosition<i32> {
        self.maybe_wait_on_main(|delegate| delegate.surface_position())
    }

    fn outer_position(&self) -> Result<dpi::PhysicalPosition<i32>, RequestError> {
        self.maybe_wait_on_main(|delegate| delegate.outer_position())
    }

    fn set_outer_position(&self, position: Position) {
        self.maybe_wait_on_main(|delegate| delegate.set_outer_position(position));
    }

    fn surface_size(&self) -> dpi::PhysicalSize<u32> {
        self.maybe_wait_on_main(|delegate| delegate.surface_size())
    }

    fn request_surface_size(&self, size: Size) -> Option<dpi::PhysicalSize<u32>> {
        self.maybe_wait_on_main(|delegate| delegate.request_surface_size(size))
    }

    fn outer_size(&self) -> dpi::PhysicalSize<u32> {
        self.maybe_wait_on_main(|delegate| delegate.outer_size())
    }

    fn safe_area(&self) -> dpi::PhysicalInsets<u32> {
        self.maybe_wait_on_main(|delegate| delegate.safe_area())
    }

    fn set_min_surface_size(&self, min_size: Option<Size>) {
        self.maybe_wait_on_main(|delegate| delegate.set_min_surface_size(min_size))
    }

    fn set_max_surface_size(&self, max_size: Option<Size>) {
        self.maybe_wait_on_main(|delegate| delegate.set_max_surface_size(max_size));
    }

    fn surface_resize_increments(&self) -> Option<dpi::PhysicalSize<u32>> {
        self.maybe_wait_on_main(|delegate| delegate.surface_resize_increments())
    }

    fn set_surface_resize_increments(&self, increments: Option<Size>) {
        self.maybe_wait_on_main(|delegate| delegate.set_surface_resize_increments(increments));
    }

    fn set_title(&self, title: &str) {
        self.maybe_wait_on_main(|delegate| delegate.set_title(title));
    }

    fn set_transparent(&self, transparent: bool) {
        self.maybe_wait_on_main(|delegate| delegate.set_transparent(transparent));
    }

    fn set_blur(&self, blur: bool) {
        self.maybe_wait_on_main(|delegate| delegate.set_blur(blur));
    }

    fn set_visible(&self, visible: bool) {
        self.maybe_wait_on_main(|delegate| delegate.set_visible(visible));
    }

    fn is_visible(&self) -> Option<bool> {
        self.maybe_wait_on_main(|delegate| delegate.is_visible())
    }

    fn set_resizable(&self, resizable: bool) {
        self.maybe_wait_on_main(|delegate| delegate.set_resizable(resizable))
    }

    fn is_resizable(&self) -> bool {
        self.maybe_wait_on_main(|delegate| delegate.is_resizable())
    }

    fn set_enabled_buttons(&self, buttons: WindowButtons) {
        self.maybe_wait_on_main(|delegate| delegate.set_enabled_buttons(buttons))
    }

    fn enabled_buttons(&self) -> WindowButtons {
        self.maybe_wait_on_main(|delegate| delegate.enabled_buttons())
    }

    fn set_minimized(&self, minimized: bool) {
        self.maybe_wait_on_main(|delegate| delegate.set_minimized(minimized));
    }

    fn is_minimized(&self) -> Option<bool> {
        self.maybe_wait_on_main(|delegate| delegate.is_minimized())
    }

    fn set_maximized(&self, maximized: bool) {
        self.maybe_wait_on_main(|delegate| delegate.set_maximized(maximized));
    }

    fn is_maximized(&self) -> bool {
        self.maybe_wait_on_main(|delegate| delegate.is_maximized())
    }

    fn set_fullscreen(&self, fullscreen: Option<Fullscreen>) {
        self.maybe_wait_on_main(|delegate| delegate.set_fullscreen(fullscreen))
    }

    fn fullscreen(&self) -> Option<Fullscreen> {
        self.maybe_wait_on_main(|delegate| delegate.fullscreen())
    }

    fn set_decorations(&self, decorations: bool) {
        self.maybe_wait_on_main(|delegate| delegate.set_decorations(decorations));
    }

    fn is_decorated(&self) -> bool {
        self.maybe_wait_on_main(|delegate| delegate.is_decorated())
    }

    fn set_window_level(&self, level: WindowLevel) {
        self.maybe_wait_on_main(|delegate| delegate.set_window_level(level));
    }

    fn set_window_icon(&self, window_icon: Option<Icon>) {
        self.maybe_wait_on_main(|delegate| delegate.set_window_icon(window_icon));
    }

    fn request_ime_update(&self, request: ImeRequest) -> Result<(), ImeRequestError> {
        self.maybe_wait_on_main(|delegate| delegate.request_ime_update(request))
    }

    fn ime_capabilities(&self) -> Option<ImeCapabilities> {
        self.maybe_wait_on_main(|delegate| delegate.ime_capabilities())
    }

    fn focus_window(&self) {
        self.maybe_wait_on_main(|delegate| delegate.focus_window());
    }

    fn has_focus(&self) -> bool {
        self.maybe_wait_on_main(|delegate| delegate.has_focus())
    }

    fn request_user_attention(&self, request_type: Option<UserAttentionType>) {
        self.maybe_wait_on_main(|delegate| delegate.request_user_attention(request_type));
    }

    fn set_theme(&self, theme: Option<Theme>) {
        self.maybe_wait_on_main(|delegate| delegate.set_theme(theme));
    }

    fn theme(&self) -> Option<Theme> {
        self.maybe_wait_on_main(|delegate| delegate.theme())
    }

    fn set_content_protected(&self, protected: bool) {
        self.maybe_wait_on_main(|delegate| delegate.set_content_protected(protected));
    }

    fn title(&self) -> String {
        self.maybe_wait_on_main(|delegate| delegate.title())
    }

    fn set_cursor(&self, cursor: Cursor) {
        self.maybe_wait_on_main(|delegate| delegate.set_cursor(cursor));
    }

    fn set_cursor_position(&self, position: Position) -> Result<(), RequestError> {
        self.maybe_wait_on_main(|delegate| delegate.set_cursor_position(position))
    }

    fn set_cursor_grab(
        &self,
        mode: winit_core::window::CursorGrabMode,
    ) -> Result<(), RequestError> {
        self.maybe_wait_on_main(|delegate| delegate.set_cursor_grab(mode))
    }

    fn set_cursor_visible(&self, visible: bool) {
        self.maybe_wait_on_main(|delegate| delegate.set_cursor_visible(visible))
    }

    fn drag_window(&self) -> Result<(), RequestError> {
        self.maybe_wait_on_main(|delegate| delegate.drag_window())
    }

    fn drag_resize_window(
        &self,
        direction: winit_core::window::ResizeDirection,
    ) -> Result<(), RequestError> {
        Ok(self.maybe_wait_on_main(|delegate| delegate.drag_resize_window(direction))?)
    }

    fn show_window_menu(&self, position: Position) {
        self.maybe_wait_on_main(|delegate| delegate.show_window_menu(position))
    }

    fn set_cursor_hittest(&self, hittest: bool) -> Result<(), RequestError> {
        self.maybe_wait_on_main(|delegate| delegate.set_cursor_hittest(hittest));
        Ok(())
    }

    fn current_monitor(&self) -> Option<CoreMonitorHandle> {
        self.maybe_wait_on_main(|delegate| {
            delegate.current_monitor().map(|monitor| CoreMonitorHandle(Arc::new(monitor)))
        })
    }

    fn available_monitors(&self) -> Box<dyn Iterator<Item = CoreMonitorHandle>> {
        self.maybe_wait_on_main(|delegate| {
            Box::new(
                delegate
                    .available_monitors()
                    .into_iter()
                    .map(|monitor| CoreMonitorHandle(Arc::new(monitor))),
            )
        })
    }

    fn primary_monitor(&self) -> Option<CoreMonitorHandle> {
        self.maybe_wait_on_main(|delegate| {
            delegate.primary_monitor().map(|monitor| CoreMonitorHandle(Arc::new(monitor)))
        })
    }

    fn rwh_06_display_handle(&self) -> &dyn rwh_06::HasDisplayHandle {
        self
    }

    fn rwh_06_window_handle(&self) -> &dyn rwh_06::HasWindowHandle {
        self
    }
}

define_class!(
    #[unsafe(super(NSWindow, NSResponder, NSObject))]
    #[name = "WinitWindow"]
    #[derive(Debug)]
    pub struct WinitWindow;

    /// This documentation attribute makes rustfmt work for some reason?
    impl WinitWindow {
        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main_window(&self) -> bool {
            trace_scope!("canBecomeMainWindow");
            true
        }

        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            trace_scope!("canBecomeKeyWindow");
            true
        }
    }
);

define_class!(
    #[unsafe(super(NSPanel, NSWindow, NSResponder, NSObject))]
    #[name = "WinitPanel"]
    #[derive(Debug)]
    pub struct WinitPanel;

    /// This documentation attribute makes rustfmt work for some reason?
    impl WinitPanel {
        // although NSPanel can become key window
        // it doesn't if window doesn't have NSWindowStyleMask::Titled
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            trace_scope!("canBecomeKeyWindow");
            true
        }
    }
);

pub(super) fn window_id(window: &NSWindow) -> WindowId {
    WindowId::from_raw(window as *const _ as usize)
}

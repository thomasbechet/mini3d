#ifndef MINI3D_H
#define MINI3D_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum mini3d_button_state {
  MINI3D_BUTTON_STATE_PRESSED,
  MINI3D_BUTTON_STATE_RELEASED,
} mini3d_button_state;

typedef struct mini3d_app {
  void *_0;
} mini3d_app;

typedef struct mini3d_renderer {
  void *_0;
} mini3d_renderer;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct mini3d_app *mini3d_app_new(void);

void mini3d_app_delete(struct mini3d_app *app);

void mini3d_app_progress(struct mini3d_app *app);

void mini3d_app_render(struct mini3d_app *app, struct mini3d_renderer *renderer);

void mini3d_app_push_close_requested(struct mini3d_app *app);

void mini3d_app_push_input_button(struct mini3d_app *app,
                                  const char *name,
                                  enum mini3d_button_state state);

void mini3d_app_push_input_axis(struct mini3d_app *app, const char *name, float value);

void mini3d_app_push_input_cursor_move(struct mini3d_app *app, const float *delta);

void mini3d_app_push_input_cursor_position(struct mini3d_app *app,
                                           float x,
                                           float y,
                                           uint32_t width,
                                           uint32_t height);

struct mini3d_renderer *mini3d_renderer_new_wgpu_win32(void *hinstance, void *hwnd);

struct mini3d_renderer *mini3d_renderer_new_wgpu_xlib(uint64_t window, void *display);

void mini3d_renderer_delete(struct mini3d_renderer *renderer);

bool mini3d_renderer_present(struct mini3d_renderer *renderer);

void mini3d_renderer_resize(struct mini3d_renderer *renderer, uint32_t width, uint32_t height);

void mini3d_renderer_recreate(struct mini3d_renderer *renderer);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* MINI3D_H */

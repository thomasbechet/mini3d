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

typedef struct mini3d_application {
  void *_0;
} mini3d_application;

typedef struct mini3d_event {
  void *_0;
} mini3d_event;

typedef struct mini3d_renderer {
  void *_0;
} mini3d_renderer;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct mini3d_application *mini3d_app_new(void);

void mini3d_application_delete(struct mini3d_application *app);

bool mini3d_application_progress(struct mini3d_application *app,
                                 struct mini3d_event *events,
                                 struct mini3d_renderer *renderer);

struct mini3d_event *mini3d_event_new(void);

void mini3d_event_delete(struct mini3d_event *event);

void mini3d_event_push_input_button(struct mini3d_event *event,
                                    const char *name,
                                    enum mini3d_button_state state);

void mini3d_event_push_input_axis(struct mini3d_event *event, const char *name, float value);

void mini3d_event_push_input_cursor_move(struct mini3d_event *event, const float *delta);

void mini3d_event_push_input_cursor_position(struct mini3d_event *event,
                                             float x,
                                             float y,
                                             uint32_t width,
                                             uint32_t height);

struct mini3d_renderer *mini3d_renderer_new_wgpu_win32(void *hinstance, void *hwnd);

struct mini3d_renderer *mini3d_renderer_new_wgpu_xlib(unsigned long window, void *display);

void mini3d_renderer_delete(struct mini3d_renderer *renderer);

bool mini3d_renderer_render(struct mini3d_renderer *renderer, const struct mini3d_application *app);

void mini3d_renderer_resize(struct mini3d_renderer *renderer, uint32_t width, uint32_t height);

void mini3d_renderer_recreate(struct mini3d_renderer *renderer);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* MINI3D_H */

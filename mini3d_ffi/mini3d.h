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

typedef struct mini3d_app_events {
  void *_0;
} mini3d_app_events;

typedef struct mini3d_app_requests {
  void *_0;
} mini3d_app_requests;

typedef struct mini3d_renderer {
  void *_0;
} mini3d_renderer;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct mini3d_app *mini3d_app_new(void);

void mini3d_app_delete(struct mini3d_app *app);

bool mini3d_app_progress(struct mini3d_app *app,
                         struct mini3d_app_events *events,
                         struct mini3d_app_requests *requests,
                         struct mini3d_renderer *renderer);

struct mini3d_app_events *mini3d_app_events_new(void);

void mini3d_app_events_delete(struct mini3d_app_events *event);

void mini3d_app_events_push_input_button(struct mini3d_app_events *event,
                                         unsigned long id,
                                         enum mini3d_button_state state);

void mini3d_app_events_push_input_axis(struct mini3d_app_events *event,
                                       unsigned long id,
                                       float value);

void mini3d_app_events_push_input_mouse_move(struct mini3d_app_events *event, const float *delta);

void mini3d_app_events_push_input_mouse_position(struct mini3d_app_events *event,
                                                 float x,
                                                 float y,
                                                 uint32_t width,
                                                 uint32_t height);

struct mini3d_renderer *mini3d_renderer_new_wgpu_win32(void *hinstance, void *hwnd);

struct mini3d_renderer *mini3d_renderer_new_wgpu_xlib(unsigned long window, void *display);

void mini3d_renderer_delete(struct mini3d_renderer *renderer);

bool mini3d_renderer_render(struct mini3d_renderer *renderer, const struct mini3d_app *app);

void mini3d_renderer_resize(struct mini3d_renderer *renderer, uint32_t width, uint32_t height);

void mini3d_renderer_recreate(struct mini3d_renderer *renderer);

struct mini3d_app_requests *mini3d_app_requests_new(void);

void mini3d_app_requests_delete(struct mini3d_app_requests *requests);

bool mini3d_app_requests_shutdown(struct mini3d_app_requests *requests);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* MINI3D_H */

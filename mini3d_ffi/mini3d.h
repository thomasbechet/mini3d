#ifndef MINI3D_H
#define MINI3D_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum mini3d_action_state {
  MINI3D_ACTION_STATE_PRESSED,
  MINI3D_ACTION_STATE_RELEASED,
} mini3d_action_state;

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

typedef struct mini3d_input_database {
  uint64_t *actions;
  uint32_t action_count;
  uint64_t *axis;
  uint32_t axis_count;
  uint64_t *groups;
  uint32_t group_count;
} mini3d_input_database;

typedef struct mini3d_input_action {
  char name[128];
  uint64_t group;
} mini3d_input_action;

typedef struct mini3d_input_axis {
  char name[128];
  uint64_t group;
} mini3d_input_axis;

typedef struct mini3d_input_group {
  char name[128];
} mini3d_input_group;

typedef struct mini3d_utils_import_image_info {
  const char *source;
  const char *name;
} mini3d_utils_import_image_info;

typedef struct mini3d_utils_import_model_info {
  const char *obj_source;
  const char *name;
  bool flat_normals;
} mini3d_utils_import_model_info;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct mini3d_app *mini3d_app_new(void);

void mini3d_app_delete(struct mini3d_app *app);

bool mini3d_app_progress(struct mini3d_app *app,
                         struct mini3d_app_events *events,
                         struct mini3d_app_requests *requests,
                         struct mini3d_renderer *renderer,
                         float delta_time);

struct mini3d_app_events *mini3d_app_events_new(void);

void mini3d_app_events_delete(struct mini3d_app_events *event);

void mini3d_app_events_push_input_action(struct mini3d_app_events *event,
                                         unsigned long id,
                                         enum mini3d_action_state state);

void mini3d_app_events_push_input_axis(struct mini3d_app_events *event,
                                       unsigned long id,
                                       float value);

struct mini3d_input_database mini3d_input_database_read(const struct mini3d_app *app);

void mini3d_input_database_free(struct mini3d_input_database *inputs);

int mini3d_input_database_get_action(const struct mini3d_app *app,
                                     uint64_t id,
                                     struct mini3d_input_action *action);

int mini3d_input_database_get_axis(const struct mini3d_app *app,
                                   uint64_t id,
                                   struct mini3d_input_axis *axis);

int mini3d_input_database_get_group(const struct mini3d_app *app,
                                    uint64_t id,
                                    struct mini3d_input_group *group);

int mini3d_utils_import_image(const struct mini3d_utils_import_image_info *info,
                              struct mini3d_app_events *events);

int mini3d_utils_import_model(const struct mini3d_utils_import_model_info *info,
                              struct mini3d_app_events *events);

struct mini3d_renderer *mini3d_renderer_new_wgpu_win32(void *hinstance, void *hwnd);

struct mini3d_renderer *mini3d_renderer_new_wgpu_xlib(unsigned long window, void *display);

void mini3d_renderer_delete(struct mini3d_renderer *renderer);

bool mini3d_renderer_render(struct mini3d_renderer *renderer, const struct mini3d_app *app);

void mini3d_renderer_resize(struct mini3d_renderer *renderer, uint32_t width, uint32_t height);

void mini3d_renderer_recreate(struct mini3d_renderer *renderer);

struct mini3d_app_requests *mini3d_app_requests_new(void);

void mini3d_app_requests_delete(struct mini3d_app_requests *requests);

bool mini3d_app_requests_shutdown(const struct mini3d_app_requests *requests);

bool mini3d_app_requests_reload_bindings(const struct mini3d_app_requests *requests);

void mini3d_app_requests_reset(struct mini3d_app_requests *requests);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* MINI3D_H */

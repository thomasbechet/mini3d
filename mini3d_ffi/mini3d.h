#ifndef MINI3D_H
#define MINI3D_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct mini3d_app_events {
  void *_0;
} mini3d_app_events;

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

struct mini3d_app_events *mini3d_app_events_new(void);

void mini3d_app_events_delete(struct mini3d_app_events *event);

void mini3d_app_events_push_input_action(struct mini3d_app_events *event,
                                         uint64_t uid,
                                         bool pressed);

void mini3d_app_events_push_input_axis(struct mini3d_app_events *event, uint64_t uid, float value);

int mini3d_utils_import_image(const struct mini3d_utils_import_image_info *info,
                              struct mini3d_app_events *events);

int mini3d_utils_import_model(const struct mini3d_utils_import_model_info *info,
                              struct mini3d_app_events *events);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* MINI3D_H */

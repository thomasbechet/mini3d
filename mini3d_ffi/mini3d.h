#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum mini3d_button_state {
  MINI3D_BUTTON_STATE_PRESSED,
  MINI3D_BUTTON_STATE_RELEASED,
} mini3d_button_state;

typedef struct mini3d_app {
  uint8_t _data[0];
} mini3d_app;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct mini3d_app *mini3d_app_new(void);

void mini3d_app_delete(struct mini3d_app *app);

void mini3d_app_push_close_requested(struct mini3d_app *app);

void mini3d_app_push_input_button(struct mini3d_app *app,
                                  const char *name,
                                  enum mini3d_button_state state);

void mini3d_app_push_input_axis(struct mini3d_app *app, const char *name, float value);

void mini3d_app_push_input_cursor_move(struct mini3d_app *app, const float *delta);

void mini3d_app_push_input_cursor_position(struct mini3d_app *app, const float *position);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

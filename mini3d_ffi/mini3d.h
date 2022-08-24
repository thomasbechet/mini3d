#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum ButtonState {
  Pressed,
  Released,
} ButtonState;

typedef struct App {
  uint8_t private_[0];
} App;

struct App *mini3d_app_new(void);

void mini3d_app_delete(struct App *app);

void mini3d_app_push_close_requested(struct App *app);

void mini3d_app_push_input_button(struct App *app, const char *name, enum ButtonState state);

void mini3d_app_push_input_axis(struct App *app, const char *name, float value);

void mini3d_app_push_input_cursor_move(struct App *app, const float *delta);

void mini3d_app_push_input_cursor_position(struct App *app, const float *position);

// MIT/Apache2 License

#ifndef CGDIPLUS_H
#define CGDIPLUS_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>
#include <windef.h>

/* Purpose */

// This provides an easy C wrapper around the Gdi+ Win32 header. This has the
// purpose of providing access to GDI+ functions to Rust gui-tools.

/* Structs */

//! Basic four-element color.
typedef struct {
  uint8_t r;
  uint8_t g;
  uint8_t b;
  uint8_t a;
} GDIPColor;

//! A graphics object is designed primarily as a pointer to the native graphics
//! object, as well as a result. We can have both of these.
typedef struct {
  void *native_graphics;
  ptrdiff_t last_status;
} GDIPGraphics;

//! A pen is designed in a similar manner.
typedef struct {
  void *native_pen;
  ptrdiff_t last_status;
} GDIPPen;

//! Same with the brush.
typedef struct {
  void *native_brush;
  ptrdiff_t last_status;
} GDIPBrush;

/* Functions */

//! Initialize the GDI, returns a startup token.
int initialize_gdiplus(ULONG_PTR *startup_token);
//! Uninitialize the GDI, given the startup token.
void done_gdiplus(ULONG_PTR startup_token);

//! Get a pointer to the last thing that went wrong, if applicable.
const char *err_pointer();

//! Create a GDIPlus graphics item from an HDC.
int from_hdc(HDC hDC, GDIPGraphics *graphics);

//! Dealloc a GDIPlus graphics instance.
void done_graphics(GDIPGraphics graphics);

//! Create a new pen from a color and a width.
int create_pen(GDIPColor color, uint32_t width, GDIPPen *pen);
//! Dealloc a pen.
void done_pen(GDIPPen pen);

//! Create a new brush from a color.
int create_brush(GDIPColor color, GDIPBrush *brush);
//! Dealloc a brush.
void done_brush(GDIPBrush brush);

//! Draw a line from one point to another.
int draw_line(GDIPGraphics *graphics, const GDIPPen *pen, int x1, int y1,
              int x2, int y2);
//! Draw a rectangle.
int draw_rectangle(GDIPGraphics *graphics, const GDIPPen *pen, int x, int y,
                   unsigned int width, unsigned int height);
//! Draw an arc.
int draw_arc(GDIPGraphics *graphics, const GDIPPen *pen, int rectleft,
             int recttop, unsigned int rectwidth, unsigned int rectheight,
             float start_angle, float end_angle);
//! Draw an ellipse.
int draw_ellipse(GDIPGraphics *graphics, const GDIPPen *pen, int rectleft,
                 int recttop, unsigned int rectwidth, unsigned int rectheight);

//! Fill a rectangle.
int fill_rectangle(GDIPGraphics *graphics, const GDIPBrush *brush, int x, int y,
                   unsigned int width, unsigned int height);
//! Fill an arc.
int fill_arc(GDIPGraphics *graphics, const GDIPBrush *brush, int rectleft,
             int recttop, unsigned int rectwidth, unsigned int rectheight,
             float start_angle, float end_angle);
//! Fill an ellispe.
int fill_ellipse(GDIPGraphics *graphics, const GDIPBrush *brush, int rectleft,
                 int recttop, unsigned int rectwidth, unsigned int rectheight);

#ifdef __cplusplus
}
#endif

#endif  // CGDIPLUS_H

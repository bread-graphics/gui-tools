// MIT/Apache2 License

// Implementations of the cgdiplus.h header, using C++.

#include "cgdiplus.h"

#include <windows.h>
#include <gdiplus.h>

//! List of errors.
const char *error_list[] = {
    "GDI+ threw an exception",
    "GDI+ function returned non-OK status",
    "startup_token pointer is null",
};

size_t current_error = SIZE_MAX;

#define EXCEPTION_OUT \
  current_error = 0;  \
  return 0;
#define STATUS_OUT   \
  current_error = 1; \
  return 0;

//! Helper to convert a C color to a GDI+ color.
inline Gdiplus::Color cvt_clr(GDIPColor clr) {
  return Gdiplus::Color(clr.r, clr.g, clr.b, clr.a);
}

//! Get the current error.
extern "C" const char *err_pointer() { return error_list[current_error]; }

//! Initialize GDI+
extern "C" int initialize_gdiplus(ULONG_PTR *startup_token) {
  Gdiplus::GdiplusStartupInput startup_input;
  if
    (Gdiplus::GdiplusStartup(startup_token, &startup_input, nullptr) !=
        Status::Ok) {
      STATUS_OUT;
    }
  return 1;
}

//! Uninitialize GDI+
extern "C" void done_gdiplus(ULONG_PTR startup_token) {
  Gdiplus::GdiplusShutdown(startup_token);
}

//! Create a new graphics object from an HDC.
extern "C" int from_hdc(HDC hDC, GDIPGraphics *graphics) {
  // Write the graphics object the Gdiplus equivalent.
  Gdiplus::Graphics *g = (Gdiplus::Graphics *)graphics;
  *g = Gdiplus::Graphics(hDC);
  return 1;
}

//! Dealloc a graphics object.
extern "C" void done_graphics(GDIPGraphics graphics) {
  // Transmute the GDIPGraphics to this.
  Gdiplus::Graphics g = (Gdiplus::Graphics)graphics;
  // Let it go out of scope.
}

//! Create a new pen from a color and a width.
extern "C" int create_pen(GDIPColor color, uint32_t width, GDIPPen *pen) {
  Gdiplus::Pen *p = (Gdiplus::Pen *)(pen);
  *p = Pen(cvt_clr(color), width);
  return 1;
}

//! Dealloc a pen.
extern "C" void done_pen(GDIPPen pen) { Gdiplus::Pen p = (Gdiplus::Pen)pen; }

//! Create a new brush.
extern "C" int create_brush(GDIPColor color, GDIBrush *brush) {
  Gdiplus::Brush *b = (Gdiplus::Brush *)brush;
  *b = Brush(cvt_clr(color));
  return 1;
}

//! Dealloc a brush.
extern "C" void done_brush(GDIPBrush brush) {
  Gdiplus::Brush b = (GDIPBrush)brush;
}

//! Draw a line from one point or another.
extern "C" int draw_line(GDIPGraphics *graphics, const GDIPPen *pen, int x1,
                         int y1, int x2, int y2) {
  static_cast<Gdiplus::Graphics *>(graphics)->DrawLine(
      static_cast<const Gdiplus::Pen *>(pen), x1, y1, x2, y2);
  return 1;
}

extern "C" int draw_rectangle(GDIPGraphics *graphics, const GDIPPen *pen, int x,
                              int y, unsigned int width, unsigned int height) {
  static_cast<Gdiplus::Graphics *>(graphics)->DrawRectangle(
      static_cast<const Gdiplus::Pen *>(pen), x, y, width, height);
  return 1;
}

extern "C" int draw_arc(GDIPGraphics *graphics, const GDIPPen *pen,
                        int rectleft, int recttop, unsigned int rectwidth,
                        unsigned int rectheight, float start_angle,
                        float end_angle) {
  static_cast<Gdiplus::Graphics *>(graphics)->DrawArc(
      static_cast<const Gdiplus::Pen *>(pen), rectleft, recttop, rectwidth,
      rectheight, start_angle, end_angle - start_angle);
  return 1;
}

extern "C" int draw_ellipse(GDIPGraphics *graphics, const GDIPPen *pen,
                            int rectleft, int recttop, unsigned int rectwidth,
                            unsigned int rectheight) {
  static_cast<Gdiplus::Graphics *>(graphics)->DrawEllipse(
      static_cast<const Gdiplus::Pen *>(pen), rectleft, recttop, rectwidth,
      rectheight);
  return 1;
}

extern "C" int fill_rectangle(GDIPGraphics *graphics, const GDIPBrush *brush,
                              int rectleft, int recttop, unsigned int rectwidth,
                              unsigned int rectheight) {
  static_cast<Gdiplus::Graphics *>(graphics)->FillRectangle(
      static_cast<const Gdiplus::Brush *>(brush), rectleft, recttop, rectwidth,
      rectheight);
  return 1;
}

extern "C" int fill_arc(GDIPGraphics *graphics, const GDIPBrush *brush,
                        int rectleft, int recttop, unsigned int rectwidth,
                        unsigned int rectheight, float start_angle,
                        float end_angle) {
  static_cast<Gdiplus::Graphics *>(graphics)->FillPie(
      static_cast<const Gdiplus::Brush *>(brush), rectleft, recttop, rectwidth,
      rectheight, start_angle, end_angle - start_angle);
  return 1;
}

extern "C" int fill_ellipse(GDIPGraphics *graphics, const GDIPBrush *brush,
                            int rectleft, int recttop, unsigned int rectwidth,
                            unsigned int rectheight) {
  static_cast<Gdiplus::Graphics *>(graphics)->FillEllipse(
      static_cast<const Gdiplus::Brush *>(brush), rectleft, recttop, rectwidth,
      rectheight);
  return 1;
}

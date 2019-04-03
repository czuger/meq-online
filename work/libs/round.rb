def round(source_image, radius =35)

  cols = source_image.columns
  rows = source_image.rows

  # Set a transparent background: pixels that are transparent will be
  # discarded from the source image.
  mask = Magick::Image.new(cols, rows) {self.background_color = 'transparent'}

  # Create a white rectangle with rounded corners. This will become the
  # mask for the area you want to retain in the original image.
  Magick::Draw.new.stroke('none').stroke_width(0).fill('white').
      roundrectangle(0, 0, cols, rows, radius, radius).
      draw(mask)

  # Apply the mask and write it out
  source_image.composite!(mask, 0, 0, Magick::CopyOpacityCompositeOp)
  source_image
end
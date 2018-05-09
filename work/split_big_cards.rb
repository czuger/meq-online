require 'rmagick'
require 'fileutils'

card_width=407
card_height=584

card_x_decal = 2

out_directory = 'sauron/plots/adv'
file = 'sauron/adv_plot_cards.jpg'

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

img = Magick::Image.read(file)[0]
card_id = 0

0.upto(2) do |line|
  0.upto(9) do |col|
    y = line*card_height
    x = col*(card_width+card_x_decal)

    crop = img.crop(x, y, card_width, card_height)
    crop = round( crop )
    crop.write("#{out_directory}/#{card_id}.png")
    card_id += 1
  end
end
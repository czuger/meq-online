require 'rmagick'
require 'fileutils'

card_width=405
card_height=586

radius = 35

%w( heroes sauron ).each do |file|
  img = Magick::Image.read("#{file}.jpg")[0]

  i = 0

  0.upto(4) do |col|
    y = 0
    x = col*card_width + (col*5)

    crop = img.crop(x, y, card_width, card_height)
    crop.rotate!(-90)

    # Set a transparent background: pixels that are transparent will be
    # discarded from the source image.
    mask = Magick::Image.new(card_height, card_width){ self.background_color = 'transparent' }

    # Create a white rectangle with rounded corners. This will become the
    # mask for the area you want to retain in the original image.
    Magick::Draw.new.stroke('none').stroke_width(0).fill('white').
        roundrectangle(0, 0, card_height, card_width-2, radius, radius ).
        draw(mask)

    # Apply the mask and write it out
    crop.composite!(mask, Magick::CenterGravity, Magick::CopyOpacityCompositeOp)

    crop.write("output/#{file}/#{file}_#{col}.png")
  end

end


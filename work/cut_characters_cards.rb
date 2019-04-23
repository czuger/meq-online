require 'rmagick'
require 'fileutils'

require_relative 'libs/round'

directory = 'raw_pics/characters'

Dir.entries( directory ).each do |f|
  next if f == '..' || f == '.'
  p = directory + '/' + f
  p p
  img = Magick::Image.read(p)[0]
  new_img = img.crop(0, 0, 92, 92)
  new_img = round( new_img, 20 )
  new_img.resize_to_fit!( 67, 67 )

  new_image_name = '../app/assets/images/characters/' + File.basename( f, '.jpg' ) + '.png'

  new_img.write( new_image_name )
end




require 'rmagick'
require 'fileutils'

directory = 'characters'

Dir.entries( directory ).each do |f|
  next if f == '..' || f == '.'
  p = directory + '/' + f
  p p
  img = Magick::Image.read(p)[0]
  new_img = img.crop(0, 0, 92, 92)
  new_img.write(directory + '/small_' + File.basename( f ) )
end




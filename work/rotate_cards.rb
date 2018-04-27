require 'rmagick'
require 'fileutils'

Dir.entries( 'monsters/monsters' ).each do |f|
  break if f == '..'
  p = 'monsters/monsters/' + f
  p p
  img = Magick::Image.read(p)[0]
  img.rotate!(-90)
  img.write(p)
end




require 'rmagick'
require 'fileutils'

def circle_crop( im )
  circle = Magick::Image.new 275, 275
  gc = Magick::Draw.new
  gc.fill 'black'
  gc.circle 275/2, 275/2, 275/2, 0
  gc.draw circle

  mask = circle.blur_image(0,1).negate

  mask.matte = false
  im.matte = true
  im.composite(mask, Magick::CenterGravity, Magick::CopyOpacityCompositeOp)
end

map = Magick::Image.read('../app/assets/images/map.jpg')[0]

x = 633
w = 275
l = 275

black_serpent = map.crop(x, 1895, w, l)
black_serpent = circle_crop(black_serpent)
black_serpent.resize_to_fit!( 48, 48 )
black_serpent.write( 'monsters/tokens_processed/minions/black_serpent.png' )

mouth_of_sauron = map.crop(x, 2180, w, l)
mouth_of_sauron = circle_crop(mouth_of_sauron)
mouth_of_sauron.resize_to_fit!( 48, 48 )
mouth_of_sauron.write( 'monsters/tokens_processed/minions/mouth_of_sauron.png' )

gothmog = map.crop(x, 2465, w, l)
gothmog = circle_crop(gothmog)
gothmog.resize_to_fit!( 48, 48 )
gothmog.write( 'monsters/tokens_processed/minions/gothmog.png' )

ringwraith = map.crop(x, 2755, w, l)
ringwraith = circle_crop(ringwraith)
ringwraith.resize_to_fit!( 48, 48 )
ringwraith.write( 'monsters/tokens_processed/minions/ringwraith.png' )

witch_king = map.crop(x, 3040, w, l)
witch_king = circle_crop(witch_king)
witch_king.resize_to_fit!( 48, 48 )
witch_king.write( 'monsters/tokens_processed/minions/witch_king.png' )
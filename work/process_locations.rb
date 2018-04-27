require 'pp'
require 'yaml'

regions = {
    'Y' => 'Endwith', 'O' => 'Eriador', 'P' => 'Rhuadur', 'DB' => 'Rhoan', 'LB' => 'Gondor',
    'DG' => 'Mirkwood', 'LG' => 'Misty mountains', 'W' => 'Grey mountains', 'B' => 'Brown lands', 'Red' => 'Mordor'
}

colors = {
    'Y' => 'Yellow', 'O' => 'Orange', 'P' => 'Purple', 'DB' => 'Dark blue', 'LB' => 'Light blue',
    'DG' => 'Dark green', 'LG' => 'Light green', 'W' => 'White', 'B' => 'Brown', 'Red' => 'Red'
}

perilous_locations = [ 'The Ruins of Angmar', 'Dol Guldur', 'Barad-Dur', 'Moria', 'Mount Gundabad' ]
heavens = [ 'Fornost', 'Erebor', 'Rivendell', 'Woodland Realm', 'Minas Tirith', 'Lothlorien' ]

def symbolize( name )
  name.gsub( /[-_ ]/, '_' ).downcase.to_sym
end

File.open('meq-locref.txt','r') do |f|

  locations = {}

  f.readlines.each do |line|
    match = line.match /([\w -]+) \((\w+)\)/
    name = match[1]
    type = match[2]

    location = { name: name, region: regions[type], region_code: symbolize( regions[type] ),
                 color: colors[type], color_code: symbolize( colors[type] ),
                 perilous: perilous_locations.include?( name ), heaven: heavens.include?( name ) }
    locations[symbolize(name)] = location
  end

  File.open('../app/models/data/locations/locations.yaml','w') do |of|
    of.puts locations.to_yaml
  end
end

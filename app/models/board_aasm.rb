module BoardAasm

  def all_heroes_played?
    heroes.pluck( :turn_finished ).each do |tf|
      return false unless tf
    end
    true
  end

  def clean_heroes_played_status!
    heroes.update_all( turn_finished: false )
  end

end
class Objectives

  CARDS_PATH = 'objectives/original/'

  def self.set_objectives(board)
    board.heroes_objective = rand( 0 .. 4 )
    board.sauron_objective = rand( 0 .. 4 )
    board.save!
  end

  def self.heroes_objective_card(board)
    CARDS_PATH + 'heroes/heroes_' + board.heroes_objective if board.heroes_objective
  end

  def self.sauron_objective_card(board)
    CARDS_PATH + 'heroes/sauron_' + board.sauron_objective if board.sauron_objective
  end

end
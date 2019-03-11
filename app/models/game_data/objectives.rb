module GameData
  class Objectives

    CARDS_PATH = 'objectives/original/'

    def self.set_objectives(board)
      board.heroes_objective = rand( 0 .. 4 )
      board.sauron_objective = rand( 0 .. 4 )
      board.save!
    end

    def self.heroes_objective_card(board)
      CARDS_PATH + 'heroes/heroes_' + board.heroes_objective.to_s + '.png' if board.heroes_objective
    end

    def self.sauron_objective_card(board)
      CARDS_PATH + 'sauron/sauron_' + board.sauron_objective.to_s + '.png' if board.sauron_objective
    end

  end
end
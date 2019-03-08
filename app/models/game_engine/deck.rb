module GameEngine
  class Deck

    def initialize( user, board, actor, deck_name )
      @user = user
      @board = board
      @actor = actor
      @deck_name = deck_name
    end

    def draw_cards( nb_cards = 1)
      nb_cards = nb_cards.to_i
      @cards = @board.send( "#{@deck_name}_deck" ).shift(nb_cards)
      @actor.assign_attributes( { "drawn_#{@deck_name}_cards" =>  @actor.send( "drawn_#{@deck_name}_cards" ) + @cards } )

      @board.transaction do
        @actor.save!
        @board.save!
        @board.log!( @user, @board.sauron, "#{@deck_name}_cards.draw", { count: @cards.count } )
      end
    end

  end
end

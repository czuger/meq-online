module GameEngine
  class Deck

    DECKS_LOCATION = { deck: 'board'.freeze, discard: 'board'.freeze, drawn_cards: 'actor'.freeze, hand: 'actor'.freeze }

    def initialize( user, board, actor, deck_name )
      @user = user
      @board = board
      @actor = actor
      @deck_name = deck_name
    end

    def draw_cards( nb_cards = 1 )
      nb_cards = nb_cards.to_i

      cards = get_deck( :deck ).shift(nb_cards)
      add_cards(:drawn_cards, cards)
      remove_cards(:deck, cards)

      @board.transaction do
        @actor.save!
        @board.save!
        @board.log!( @user, @board.sauron, "#{@deck_name}_cards.draw", { count: cards.count } )
      end
    end

    def keep_cards( selected_cards, back_to_bottom_of_deck = true )
      raise 'Selected cards should not be empty' if selected_cards.empty?
      raise 'Selected cards should be included in pool cards' unless (selected_cards - @actor.send( "drawn_#{@deck_name}_cards" )).empty?

      @hand += selected_cards
      @drawn_cards -= selected_cards

      @board.transaction do

        if back_to_bottom_of_deck
          back_to_bottom_of_deck
        else
          # To discard deck
        end

        @board.save!
        @actor.save!
        @board.log!( @user, @board.sauron, "#{@deck_name}_cards.keep", { count: selected_cards.count } )
      end
    end

    private

    def back_to_bottom_of_deck
      cards_count = @drawn_cards.count
      @deck += @drawn_cards
      @drawn_cards = []

      @board.log!( @user, @board.sauron, "#{@deck_name}_cards.back_to_bottom", { count: cards_count } )
    end

    def add_cards( deck_code_name, cards )
      check_decks_location( deck_code_name )
      instance_variable_get('@'+DECKS_LOCATION[deck_code_name]).assign_attributes(
          { deck_code_to_name( deck_code_name ) => get_deck(deck_code_name) + cards } )
    end

    def remove_cards( deck_code_name, cards )
      check_decks_location( deck_code_name )
      instance_variable_get('@'+DECKS_LOCATION[deck_code_name]).assign_attributes(
          { deck_code_to_name( deck_code_name ) => get_deck(deck_code_name) - cards } )
    end

    def get_deck( deck_code_name )
      check_decks_location( deck_code_name )
      instance_variable_get('@'+DECKS_LOCATION[deck_code_name]).send(deck_code_to_name(deck_code_name))
    end

    def deck_code_to_name( deck_code_name )
      case deck_code_name
        when :deck
          "#{@deck_name}_deck".freeze
        when :discard
          "#{@deck_name}_discard".freeze
        when :drawn_cards
          "drawn_#{@deck_name}_cards".freeze
        when :hand
          "#{@deck_name}_cards"
        else
          raise "Unknown deck code name : (#{deck_code_name})"
      end
    end

    def check_decks_location(deck_code_name)
      raise "Unknown #{deck_code_name}" unless DECKS_LOCATION.keys.include?( deck_code_name )
    end

  end
end

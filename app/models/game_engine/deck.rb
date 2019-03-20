module GameEngine
  class Deck

    DECKS_LOCATION = { deck: 'board'.freeze, discard: 'board'.freeze, drawn_cards: 'actor'.freeze, hand: 'actor'.freeze }
    DISCARD_CARD_ACTIONS = [ :back_to_bottom, :discard ]

    def initialize( user, board, actor, deck_name, discard_card_action: nil )
      @user = user
      @board = board
      @actor = actor
      @deck_name = deck_name

      raise "@user can't be nil" unless @user
      raise "@board can't be nil" unless @board
      raise "@actor can't be nil" unless @actor
      raise "@deck_name can't be nil" unless @deck_name

      @discard_card_action = discard_card_action
      raise "discard_card_action can't be nil" unless @discard_card_action
      raise "discard_card_action should be included in #{DISCARD_CARD_ACTIONS}" unless DISCARD_CARD_ACTIONS.include? @discard_card_action

      @back_to_bottom = true if discard_card_action == :back_to_bottom
      @discard = true if discard_card_action == :discard
    end

    def draw_cards( nb_cards = 1 )
      nb_cards = nb_cards.to_i

      deck = get_deck( :deck )
      deck = check_and_refill_cards_deck(deck, nb_cards) if @discard

      cards = deck.shift(nb_cards)
      add_cards(:drawn_cards, cards)

      deck -= cards
      set_deck(:deck, deck)

      @board.transaction do
        @actor.save!
        @board.save!
        @board.log( @board.sauron, "#{@deck_name}_cards.draw", { count: cards.count } )
      end
    end

    def keep_cards( selected_cards, discard_card_action: nil )
      raise 'Selected cards should not be empty' if selected_cards.empty?
      raise 'Selected cards should be included in pool cards' unless (selected_cards - @actor.send( "drawn_#{@deck_name}_cards" )).empty?

      add_cards(:hand, selected_cards)
      remove_cards(:drawn_cards, selected_cards)

      @board.transaction do

        if @back_to_bottom
          set_cards_back_to_bottom_of_deck
        elsif @discard
          discard_cards
        else raise 'WOOOT ?'
        end

        @board.save!
        @actor.save!
        @board.log( @board.sauron, "#{@deck_name}_cards.keep", { count: selected_cards.count } )
      end
    end

    private

    def check_and_refill_cards_deck( deck, nb_cards )
      if nb_cards > deck.count
        # If we don't have enough cards
        deck += get_deck( :discard )
        deck.shuffle!
        empty_deck( :discard )
      end
      deck
    end

    def set_cards_back_to_bottom_of_deck
      drawn_cards = get_deck(:drawn_cards )
      cards_count = drawn_cards.count
      add_cards(:deck, drawn_cards)
      remove_cards(:drawn_cards, drawn_cards)

      @board.log( @board.sauron, "#{@deck_name}_cards.back_to_bottom", { count: cards_count } )
    end

    def discard_cards
      drawn_cards = get_deck(:drawn_cards )
      cards_count = drawn_cards.count
      add_cards(:discard, drawn_cards)
      remove_cards(:drawn_cards, drawn_cards)

      @board.log( @board.sauron, "#{@deck_name}_cards.discard", { count: cards_count } )
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

    def empty_deck( deck_code_name )
      check_decks_location( deck_code_name )
      instance_variable_get('@'+DECKS_LOCATION[deck_code_name]).assign_attributes(
          { deck_code_to_name( deck_code_name ) => [] } )
    end

    def get_deck( deck_code_name )
      check_decks_location( deck_code_name )
      instance_variable_get('@'+DECKS_LOCATION[deck_code_name]).send(deck_code_to_name(deck_code_name))
    end

    def set_deck( deck_code_name, deck )
      check_decks_location( deck_code_name )
      instance_variable_get('@'+DECKS_LOCATION[deck_code_name]).assign_attributes(
          { deck_code_to_name( deck_code_name ) => deck } )
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

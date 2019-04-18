class Combat < ApplicationRecord
  include AASM

  belongs_to :board
  belongs_to :hero, class_name: 'Actor', foreign_key: :actor_id
  belongs_to :mob

  has_many :combat_card_played_mobs
  has_many :combat_card_played_heroes

  def resolve_combat_effects( hero, mob )
    @game_data_heroes = GameData::Heroes.new
    @game_data_mobs_cards = GameData::MobsCards.new

    current_hero_card_data = @game_data_heroes.get_card_data( hero.name_code, hero.combat_card_played )
    current_mob_card_data = @game_data_mobs_cards.get_card_data( mob.attack_deck, mob.combat_card_played )

    previous_hero_card_data = @game_data_heroes.get_card_data( hero.name_code, hero.combat_cards_played.last )
    previous_mob_card_data = @game_data_mobs_cards.get_card_data( mob.attack_deck, mob.combat_cards_played.last )

    cards_params = OpenStruct.new( ch: current_hero_card_data, cm: current_mob_card_data,
                                   ph: previous_hero_card_data, pm: previous_mob_card_data )

    # call_function( previous_hero_card_data, :previous, )
  end

  def reveal_secretly_played_cards
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs ||= GameData::Mobs.new

    ch_cd = @game_data_heroes.get_card_data( hero.name_code, hero_secret_played_card )
    combat_card_played_heroes.create!( card: hero_secret_played_card, pic_path: ch_cd.pic_path, name: ch_cd.name, power: ch_cd.power,
                                       strength_cost: ch_cd.strength_cost, printed_attack: ch_cd.attack, final_attack: ch_cd.attack,
                                       printed_defense: ch_cd.defense, final_defense: ch_cd.defense, card_type: ch_cd.type )

    cm_cd = @game_data_mobs.get_card_data( hero.name_code, mob_secret_played_card )
    combat_card_played_heroes.create!( card: mob_secret_played_card, pic_path: cm_cd.pic_path, name: cm_cd.name, power: cm_cd.power,
                                       strength_cost: cm_cd.strength_cost, printed_attack: cm_cd.attack, final_attack: cm_cd.attack,
                                       printed_defense: cm_cd.defense, final_defense: cm_cd.defense, card_type: cm_cd.type )

  end

  private

  def aimed_shot( )
  end

  def call_function( moment, my_previous_card, opponent_previous_card )
    function_name = card.name.downcas.gsub( ' ', '_' )
    send( function_name, moment, params_struct )
  end

end

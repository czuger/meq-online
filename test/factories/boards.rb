FactoryBot.define do
  factory :board do
    plot_deck { (3..17).to_a }
    shadow_deck { (0..23).to_a }
    event_deck { (0..13).to_a }

    plot_discard {[]}
    shadow_discard {[]}
    event_discard {[]}

    favors {[]}

    corruption_deck {[]}
    corruption_discard {[]}

    monsters_pool_orange {[:orc, :nothing, :orc, :snaga, :snaga, :nothing, :snaga, :nothing, :snaga, :nothing]}
    monsters_pool_purple {[:nothing, :nothing, :nothing, :orc, :nothing, :snaga, :orc, :snaga, :cave_troll, :snaga]}
    monsters_pool_dark_blue {[:southron, :nothing, :nothing, :nothing, :southron, :snaga, :orc, :nothing, :snaga, :southron]}
    monsters_pool_brown {[:nothing, :cave_troll, :cave_troll, :orc, :orc, :nothing, :nothing, :cave_troll, :nothing, :cave_troll]}
    monsters_pool_dark_green {[:nothing, :nothing, :huorn, :nothing, :huorn, :snaga, :huorn, :snaga, :nothing, :orc]}

    influence { GameData::Plots.new.get(0).influence.init }
    shadow_pool { GameData::Plots.new.get(0).influence.shadow_pool }

    characters { {} }

    max_heroes_count { 2 }

    sauron_actions { [ 'command_1' ] }
  end
end

Rails.application.routes.draw do

  resources :actor, only: [] do
    resources :board_messages, only: [ :index, :new, :create ]
  end

  get 'map_coordinates/edit'
  post 'map_coordinates/update'

  resources :heroes, only: [:show ] do
    get :rest_screen
    get :rest_rest
    get :rest_heal
    get :rest_skip

    get :discard_corruption_card_screen
    post :discard_corruption_card

    get :after_rest_advance_story_marker_screen
    get :after_rest_advance_story_marker

    get :movement_screen
    post :move
    get :movement_finished

    get :argalad_power_screen

    resource :exploration, only: [:show, :update ] do
      get :next_movement
      get :next_step
    end

    get :draw_cards_screen
    post :draw_cards
    get :draw_cards_finished

    # Good example on how to have a route like : /foo/:foo_id/bar/foobar rather than /foo/:foo_id/bar/:id/foobar
    # resources :movement_preparation_steps, except: [ :show ] do
    #   collection do
    #     get :terminate
    #   end
    # end
  end

  #
  # Sauron linked actions
  #
  resources :sauron, only: [:show] do

    get :setup_screen
    get :setup_finished

    resource :shadow_cards, only: [] do
      get :play_screen
      post :play

      get :start_hero_turn_play_card_screen
      post :start_hero_turn_play_card
      get :start_hero_turn_play_card_finished

      get :draw_screen
      post :draw

      get :keep_screen
      post :keep
    end

    resource :plot_cards, only: [] do
      get :play_screen
      post :play
      get :play_finished

      get :discard_screen
      post :discard

      get :draw_screen
      post :draw

      get :keep_screen
      post :keep

      get :look_for_gollum_cards
      post :get_gollum_card
    end

    resource :sauron_actions, only: [:edit, :update] do
      get :terminate
      post :set_influence
    end

    resources :sauron_mobs, except: [:delete, :show ] do
      get :heal
    end

  end

  patch 'shadow_pools/:actor_id/update_from_map', to: 'shadow_pools#update_from_map', as: 'shadow_pools_update_from_map'

  resources :maps, only: [:edit, :show]

  resources :characters, only: [:edit,:update]

  get 'welcome', to: 'boards#welcome'

  resources :boards, only:[ :index, :new, :create ] do
    get :join, action: :join_new
    post :join

    get :inactive_actor

    resource :logs, only: [:show ]
    resources :null, only: [:index ]

    get :story_screen

    resource :combats, only: [ :show ] do
      get :combat_setup_screen
      post :combat_setup

      get :cards_loss_screen
      post :cards_loss

      get ':actor_id/play_combat_card_screen', action: :play_combat_card_screen, as: :play_combat_card_screen
      post :play_combat_card_hero
      post :play_combat_card_mob

      get ':actor_id/apply_damages', action: :apply_damages, as: :apply_damages

      get :terminate
    end
  end
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  root 'boards#index'

  match 'auth/:provider/callback', to: 'sessions#create', via: [:get, :post]
  get 'auth/failure', to: 'sessions#failure'

  resource :sessions, only: [:new, :destroy]

end
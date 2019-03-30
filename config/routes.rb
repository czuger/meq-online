Rails.application.routes.draw do

  resources :actor, only: [] do
    resources :board_messages, only: [ :index, :new, :create ]
  end

  get 'map_coordinates/edit'
  post 'map_coordinates/update'

  resources :events, only: [ :edit, :update ]

  resources :combats, only: [ :show, :new, :create, :destroy ] do
    get :hero_setup_new
    post :hero_setup_draw_cards
    post :hero_setup_increase_strength

    get :play_card_screen
    post :play_card
  end

  resources :heros, only: [ :show ] do

    get :rest_screen
    post :rest
    get :rest_finished

    get :movement_screen
    post :move
    get :movement_finished

    get :exploration_screen
    post :explore
    get :exploration_finished

    get :encounter_screen
    post :encounter
    get :encounter_finished

    get :draw_cards_screen
    post :draw_cards
    get :draw_cards_finished

    patch :take_damages
    patch :finish_turn


    resources :movement_preparation_steps, except: [ :show ] do
      collection do
        get :terminate
      end
    end
  end

  #
  # Sauron linked actions
  #
  resources :sauron, only: [:show] do

    get :setup
    get :setup_finished

    get :execute_event_card_screen

    get :movement_break_schedule_screen
    post :movement_break_schedule_add
    get :movement_break_schedule_finished

    get :story_screen
    post :story_update
    get :story_step_finished

    resource :shadow_cards, only: [] do
      get :play_screen
      post :play

      get :start_hero_turn_play_card_screen
      post :start_hero_turn_play_card
      get :start_hero_turn_play_card_finished

      get :discard_screen
      post :discard

      get :draw_screen
      post :draw

      get :keep_screen
      post :keep
    end

  end

  get 'sauron_actions/:actor_id/terminate', to: 'sauron_actions#terminate', as: 'sauron_actions_terminate'
  resources :sauron_actions, only: [:edit, :update]

  #
  # Plot cards actions
  #
  get 'plot_cards/:actor_id/play_screen', to: 'plot_cards#play_screen', as: 'plot_cards_play_screen'
  post 'plot_cards/:actor_id/play', to: 'plot_cards#play', as: 'plot_cards_play'
  get 'plot_cards/:actor_id/play_finished', to: 'plot_cards#play_finished', as: 'plot_cards_play_finished'

  get 'plot_cards/:actor_id/discard_screen', to: 'plot_cards#discard_screen', as: 'plot_cards_discard_screen'
  post 'plot_cards/:actor_id/discard', to: 'plot_cards#discard', as: 'plot_cards_discard'

  get 'plot_cards/:actor_id/draw_screen', to: 'plot_cards#draw_screen', as: 'plot_cards_draw_screen'
  post 'plot_cards/:actor_id/draw', to: 'plot_cards#draw', as: 'plot_cards_draw'

  get 'plot_cards/:actor_id/keep_screen', to: 'plot_cards#keep_screen', as: 'plot_cards_keep_screen'
  post 'plot_cards/:actor_id/keep', to: 'plot_cards#keep', as: 'plot_cards_keep'

  patch 'shadow_pools/:actor_id/update_from_map', to: 'shadow_pools#update_from_map', as: 'shadow_pools_update_from_map'

  resources :shadow_pools, only: [ :edit, :update ]

  resources :influences, only: [ :show, :edit, :update ]

  resources :maps, only: [:edit]
  resources :characters, only: [:edit,:update]
  resources :board_steps, only: [:edit,:update]

  resources :boards, only:[ :index, :new, :create ] do
    get :join, action: :join_new
    post :join

    get :inactive_actor

    resource :maps, only: [:show]

    resource :logs, only: [:show ]
    resources :heros, only: [ :index ]
  end
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  root 'boards#index'

  match 'auth/:provider/callback', to: 'sessions#create', via: [:get, :post]
  get 'auth/failure', to: 'sessions#failure'

  resource :sessions, only: [:new, :destroy]

end
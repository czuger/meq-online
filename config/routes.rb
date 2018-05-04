Rails.application.routes.draw do

  resources :combats
  resources :boards, only:[ :index, :show, :new, :create ] do

    get :join, action: :join_new
    post :join

    resources :logs, only: [:index ]
    resource :sauron, only: [:show ]

    resources :heros, only: [ :index, :show ] do
      post :draw_cards
      get :rest
      get :heal
      patch :take_damages
      post :move
    end

  end
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  root 'boards#index'

  match 'auth/:provider/callback', to: 'sessions#create', via: [:get, :post]
  get 'auth/failure', to: 'sessions#failure'

  resource :sessions, only: [:new, :destroy]

end

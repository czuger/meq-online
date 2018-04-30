Rails.application.routes.draw do

  resources :boards do
    resource :saurons, only: [:show]
    resources :logs, only: [:index ]
    resources :heros do
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

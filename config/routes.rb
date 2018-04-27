Rails.application.routes.draw do
  resources :heros
  resources :boards
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  root 'boards#index'
end

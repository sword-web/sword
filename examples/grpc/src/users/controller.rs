use async_stream::try_stream;
use std::sync::Arc;
use sword::grpc::*;
use sword::prelude::*;
use tokio::time::{self, Duration};

use crate::shared::LoggingInterceptor;
use crate::{
    shared::AuthInterceptor,
    users::{CreateUserDto, UpdateUserDto, UserRepository, proto::*},
};

#[controller(kind = Controller::Grpc, service = UserServiceServer)]
#[interceptor(LoggingInterceptor, config = "UsersController")]
#[interceptor(AuthInterceptor)]
pub struct UsersController {
    users: Arc<UserRepository>,
}

#[sword::grpc::async_trait]
impl UserService for UsersController {
    type StreamUsersStream = GrpcStream<UserItem>;

    async fn list_users(&self, _: Request<ListUsersRequest>) -> GrpcResult<ListUsersReply> {
        let users = self.users.find_all().await;

        let users = users
            .into_iter()
            .map(|u| UserItem {
                id: u.id,
                username: u.username,
            })
            .collect();

        Ok(GrpcResponse::message(ListUsersReply { users }))
    }

    async fn stream_users(
        &self,
        _: Request<StreamUsersRequest>,
    ) -> GrpcResult<Self::StreamUsersStream> {
        let users = self.users.find_all().await;

        let output = try_stream! {
            for user in users {
                yield UserItem {
                    id: user.id,
                    username: user.username,
                };

                time::sleep(Duration::from_secs(1)).await;
            }
        };

        Ok(GrpcResponse::stream(output))
    }

    async fn create_user(&self, req: Request<CreateUserRequest>) -> GrpcResult<UserReply> {
        let payload = req.into_inner();

        let dto = CreateUserDto {
            username: payload.username,
            password: payload.password,
        };

        let user = self.users.create(dto).await?;

        let user_item = UserItem {
            id: user.id,
            username: user.username.clone(),
        };

        Ok(GrpcResponse::message(UserReply {
            user: Some(user_item),
        }))
    }

    async fn get_user(&self, req: Request<GetUserRequest>) -> GrpcResult<UserReply> {
        let id = req.into_inner().id;
        let user = self.users.find_by_id(&id).await?;

        let user_item = UserItem {
            id: user.id,
            username: user.username.clone(),
        };

        Ok(GrpcResponse::message(UserReply {
            user: Some(user_item),
        }))
    }

    async fn update_user(&self, req: Request<UpdateUserRequest>) -> GrpcResult<UserReply> {
        let payload = req.into_inner();

        let dto = UpdateUserDto {
            id: payload.id,
            username: payload.username,
            password: payload.password,
        };

        let user = self.users.update(dto).await?;

        let user_item = UserItem {
            id: user.id,
            username: user.username.clone(),
        };

        Ok(GrpcResponse::message(UserReply {
            user: Some(user_item),
        }))
    }

    async fn delete_user(&self, req: Request<DeleteUserRequest>) -> GrpcResult<DeleteUserReply> {
        let id = req.into_inner().id;

        self.users.delete(&id).await?;

        Ok(GrpcResponse::message(DeleteUserReply {
            message: "User deleted".into(),
        }))
    }
}

use std::sync::Arc;

use async_stream::try_stream;
use tokio::time::{self, Duration};

use sword::grpc::*;
use sword::prelude::*;

use crate::users::proto::UserService as UserServiceApi;
use crate::{
    shared::{AuthInterceptor, LoggingInterceptor},
    users::{CreateUserDto, UpdateUserDto, UserService, proto::*},
};

#[controller(kind = Controller::Grpc, service = UserServiceServer)]
#[interceptor(LoggingInterceptor, config = "UsersController")]
#[interceptor(AuthInterceptor)]
pub struct UsersController {
    users: Arc<UserService>,
}

#[sword::grpc::async_trait]
impl UserServiceApi for UsersController {
    type StreamUsersStream = GrpcStream<UserItem>;

    async fn list_users(&self, _: Request<ListUsersRequest>) -> GrpcResult<ListUsersReply> {
        let users = self.users.find_all().await?;
        let users = users
            .iter()
            .map(UserItem::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(GrpcResponse::message(ListUsersReply { users }))
    }

    async fn stream_users(
        &self,
        _: Request<StreamUsersRequest>,
    ) -> GrpcResult<Self::StreamUsersStream> {
        let users = self.users.find_all().await?;
        let items = users
            .iter()
            .map(UserItem::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let output = try_stream! {
            for item in items {
                yield item;

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
        let user_item = UserItem::try_from(&user)?;

        Ok(GrpcResponse::message(UserReply {
            user: Some(user_item),
        }))
    }

    async fn get_user(&self, req: Request<GetUserRequest>) -> GrpcResult<UserReply> {
        let id = req.into_inner().id;
        let user = self.users.find_by_id(&id).await?;

        let user_item = UserItem::try_from(&user)?;

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
        let user_item = UserItem::try_from(&user)?;

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

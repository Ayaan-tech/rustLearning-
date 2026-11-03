use utoipa::Modify;
use utoipa::OpenApi;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};

use crate::{
    dto::{
        EmailListResponseDto, FileUploadForm, FilterEmailDto, FilterUserDto, LoginUserDto,
        NameUpdateDto, RegisterUserDto, RequestQueryDto, Response, RetrieveFileDto,
        SearchQueryByEmailDTO, UserData, UserLoginResponseDto, UserPasswordUpdateDto,
        UserReceiveFileDto, UserReceiveFileListResponseDto, UserResponseDto, UserSendFileDto,
        UserSendFileListResponseDto,
    },
    error::ErrorResponse,
    handler::{
        auth,
        file,
        file_query,
        user,
    },
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SecureShare API",
        version = "0.1.0",
        description = "Encrypted file sharing API with JWT authentication and RSA/AES encryption."
    ),
    paths(
        auth::register,
        auth::login,
        user::get_me,
        user::update_user_name,
        user::update_user_password,
        user::search_by_email,
        file::upload_file,
        file::retrieve_file,
        file_query::get_user_shared_files,
        file_query::get_receive_shared_files,
    ),
    components(
        schemas(
            RegisterUserDto,
            LoginUserDto,
            Response,
            UserLoginResponseDto,
            FilterUserDto,
            UserData,
            UserResponseDto,
            NameUpdateDto,
            UserPasswordUpdateDto,
            SearchQueryByEmailDTO,
            FilterEmailDto,
            EmailListResponseDto,
            FileUploadForm,
            RetrieveFileDto,
            RequestQueryDto,
            UserSendFileDto,
            UserSendFileListResponseDto,
            UserReceiveFileDto,
            UserReceiveFileListResponseDto,
            ErrorResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "User registration and login"),
        (name = "Users", description = "Authenticated user profile management"),
        (name = "Files", description = "Encrypted file upload and retrieval"),
        (name = "Lists", description = "Sent and received file listings"),
    ),
    security(())
)]
pub struct ApiDoc;
